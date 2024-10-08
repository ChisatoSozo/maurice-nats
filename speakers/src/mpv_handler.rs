use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    fbs::{construct_speaker_list_event_message, construct_volume_changed_event_message},
    PlayContent, SpeakerCommand, SpeakerCommandContent, SpeakerListQuery,
};

use alsa::{device_name::HintIter, Direction};
use serde_json::{json, Value};

fn send_command(
    stream: &mut UnixStream,
    command: &str,
    property: &str,
    value: &str,
) -> Result<Value, String> {
    //random int32 as request_id
    let rand_int: i32 = rand::random();
    let request_id = rand_int;
    let command = json!({
        "command": [command, property, value],
        "request_id": request_id
    });
    writeln!(stream, "{}", command.to_string()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    read_response_with_id(stream, &request_id)
}

fn send_command_single_arg(
    stream: &mut UnixStream,
    command: &str,
    arg: &str,
) -> Result<Value, String> {
    let rand_int: i32 = rand::random();
    let request_id = rand_int;
    let command = json!({
        "command": [command, arg],
        "request_id": request_id
    });
    writeln!(stream, "{}", command.to_string()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    read_response_with_id(stream, &request_id)
}

fn get_property(stream: &mut UnixStream, property: &str) -> Result<Value, String> {
    let rand_int: i32 = rand::random();
    let request_id = rand_int;
    let command = json!({
        "command": ["get_property", property],
        "request_id": request_id
    });
    writeln!(stream, "{}", command.to_string()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let response = read_response_with_id(stream, &request_id)?;
    Ok(response["data"].clone())
}

fn read_response_with_id(stream: &mut UnixStream, request_id: &i32) -> Result<Value, String> {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let parsed: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if parsed["request_id"] == *request_id {
            return Ok(parsed);
        }
    }
    Err("Response with matching request_id not found".to_string())
}

struct SpeakerState {
    music_volume: f32,
    mpv_process: Option<std::process::Child>,
    mpv_sock: Option<UnixStream>,
}

pub struct MpvHandler {
    state: HashMap<String, SpeakerState>,
}

pub fn list_pcm_devices(direction: Direction) -> Vec<String> {
    let hints = HintIter::new_str(None, "pcm").unwrap();
    let mut devices = Vec::new();
    for hint in hints {
        // Filter out unwanted virtual devices and focus on hw/plughw only
        let name = hint.name.unwrap_or_default();
        let desc = hint.desc.unwrap_or_default();
        if (name.starts_with("plughw:"))
            && (!name.contains("NVidia"))
            && (!desc.contains(" Alt "))
            && (hint.direction.is_none()
                || hint
                    .direction
                    .map(|dir| dir == direction)
                    .unwrap_or_default())
        {
            println!("Device: {:<35} Description: {:?}", name, desc);
            devices.push(name);
        }
    }
    devices
}

impl MpvHandler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            state: HashMap::new(),
        })
    }

    pub fn handle_speaker_list_query(
        &mut self,
        _: SpeakerListQuery,
    ) -> Result<Option<Vec<u8>>, String> {
        let playback_devices = list_pcm_devices(Direction::Playback);

        //add new devices to the state, remove devices that no longer exist
        let mut devices_to_remove = Vec::new();
        for (device_id, _) in self.state.iter() {
            if !playback_devices.contains(&device_id) {
                devices_to_remove.push(device_id.clone());
            }
        }

        for device_id in devices_to_remove {
            self.state.remove(&device_id);
        }

        for device_id in &playback_devices {
            if !self.state.contains_key(device_id) {
                self.state.insert(
                    device_id.clone(),
                    SpeakerState {
                        music_volume: 100.0,
                        mpv_process: None,
                        mpv_sock: None,
                    },
                );
            }
        }

        return Ok(Some(construct_speaker_list_event_message(playback_devices)));
    }

    pub fn handle_speaker_command(
        &mut self,
        command: SpeakerCommand,
    ) -> Result<Option<Vec<u8>>, String> {
        let device_id: &str = command.device_id().ok_or("No device_id")?;

        let speaker_state = self
            .state
            .get_mut(device_id)
            .ok_or("Device does not exist")?;

        const MAX: u8 = SpeakerCommandContent::ENUM_MAX + 1 as u8;
        match command.command_type() {
            SpeakerCommandContent::NONE => {
                return Err("No command".to_string());
            }
            SpeakerCommandContent::SetMusicVolume => {
                let volume = command
                    .command_as_set_music_volume()
                    .ok_or("No volume")?
                    .volume();

                //send command to mpv
                let sock = speaker_state.mpv_sock.as_mut().ok_or("No mpv socket")?;
                send_command(sock, "set_property", "volume", &volume.to_string())
                    .map_err(|e| e.to_string())?;

                speaker_state.music_volume = volume;
                return Ok(Some(construct_volume_changed_event_message(
                    volume, device_id,
                )));
            }
            SpeakerCommandContent::TogglePause => {
                let sock = speaker_state.mpv_sock.as_mut().ok_or("No mpv socket")?;
                send_command(sock, "cycle", "pause", "").map_err(|e| e.to_string())?;
            }
            SpeakerCommandContent::Play => {
                if speaker_state.mpv_process.is_some() {
                    //terminate the current mpv process
                    let mpv_process = speaker_state.mpv_process.as_mut().ok_or("No mpv process")?;
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    //clear the mpv process and socket
                    speaker_state.mpv_process = None;
                    speaker_state.mpv_sock = None;
                }

                let play_command = command.command_as_play().ok_or("No play command")?;

                const MAX: u8 = PlayContent::ENUM_MAX + 1 as u8;
                match play_command.content_type() {
                    PlayContent::NONE => {
                        return Err("No play command".to_string());
                    }
                    PlayContent::PlayWav => {
                        return Err("PlayWav not implemented".to_string());
                    }
                    PlayContent::PlayYoutube => {
                        //play the youtube video
                        let url = play_command
                            .content_as_play_youtube()
                            .ok_or("No youtube url")?
                            .url()
                            .ok_or("URL is None")?;
                        let mpv_process = std::process::Command::new("mpv")
                            .arg(url) // The YouTube URL
                            .arg("--no-video") // Skip video, play audio only
                            .arg(format!("--audio-device=alsa/{device_id}")) // The audio device to use
                            .arg(format!("--input-ipc-server=/tmp/mpv-socket-{}", device_id))
                            .spawn()
                            .map_err(|e| e.to_string())?;

                        let socket_path = format!("/tmp/mpv-socket-{}", device_id);
                        let timeout = Duration::from_secs(5); // 5 second timeout
                        let start_time = Instant::now();

                        let mut mpv_sock = UnixStream::connect(&socket_path)
                            .map_err(|e| e.to_string())
                            .map_err(|e| e.to_string());

                        while start_time.elapsed() < timeout {
                            match mpv_sock {
                                Ok(_) => break,
                                Err(_) => {}
                            }
                            mpv_sock = UnixStream::connect(&socket_path)
                                .map_err(|e| e.to_string())
                                .map_err(|e| e.to_string());
                            sleep(Duration::from_millis(100));
                        }

                        if start_time.elapsed() >= timeout {
                            return Err("Timeout waiting for MPV socket".to_string());
                        }

                        let mut mpv_sock = match mpv_sock {
                            Ok(mpv_sock) => mpv_sock,
                            Err(e) => {
                                return Err(e);
                            }
                        };

                        let command_result =
                            send_command(&mut mpv_sock, "set_property", "pause", "no")
                                .map_err(|e| e.to_string())
                                .map_err(|e| e.to_string());

                        speaker_state.mpv_process = Some(mpv_process);

                        speaker_state.mpv_sock = Some(mpv_sock);

                        match command_result {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }

                        return Ok(None);
                    }
                    PlayContent(MAX..=u8::MAX) => {
                        return Err("Unknown play command".to_string());
                    }
                }
            }
            SpeakerCommandContent::Stop => {
                if let Some(mpv_process) = &mut speaker_state.mpv_process {
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    speaker_state.mpv_process = None;
                    speaker_state.mpv_sock = None;
                }
            }
            SpeakerCommandContent::Seek => {}
            SpeakerCommandContent(MAX..=u8::MAX) => {
                // Handle the command
            }
        }

        return Ok(None);
        // match message {
        //     MpvHandlerMessage::Play(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_sock) = state.mpv_sock.borrow_mut() {
        //                 match send_command(mpv_sock, "set_property", "pause", "no") {
        //                     Ok(_) => {}
        //                     Err(e) => return MpvHandlerResponse::Error(e),
        //                 }

        //                 return MpvHandlerResponse::Ok;
        //             }

        //             //if the queue is not empty, play the first song
        //             if !state.queue.is_empty() {
        //                 let song = state
        //                     .queue
        //                     .get(0)
        //                     .ok_or("no song")
        //                     .map_err(|e| MpvHandlerResponse::Error(e.to_string()));
        //                 let song = match song {
        //                     Ok(song) => song,
        //                     Err(e) => return e,
        //                 };
        //                 let mpv_process = std::process::Command::new("mpv")
        //                     .arg("--no-video")
        //                     .arg(format!("--input-ipc-server=/tmp/mpv-socket-{}", id))
        //                     .arg(&song.url)
        //                     .spawn()
        //                     .map_err(|e| e.to_string())
        //                     .map_err(|e| MpvHandlerResponse::Error(e));
        //                 let mpv_process = match mpv_process {
        //                     Ok(mpv_process) => mpv_process,
        //                     Err(e) => return e,
        //                 };

        //                 let socket_path = format!("/tmp/mpv-socket-{}", id);
        //                 let timeout = Duration::from_secs(5); // 5 second timeout
        //                 let start_time = Instant::now();

        //                 let mut mpv_sock = UnixStream::connect(&socket_path)
        //                     .map_err(|e| e.to_string())
        //                     .map_err(|e| MpvHandlerResponse::Error(e));

        //                 while start_time.elapsed() < timeout {
        //                     match mpv_sock {
        //                         Ok(_) => break,
        //                         Err(_) => {}
        //                     }
        //                     mpv_sock = UnixStream::connect(&socket_path)
        //                         .map_err(|e| e.to_string())
        //                         .map_err(|e| MpvHandlerResponse::Error(e));
        //                     error!("Failed to connect to mpv socket, retrying...");
        //                     sleep(Duration::from_millis(100));
        //                 }

        //                 if start_time.elapsed() >= timeout {
        //                     return MpvHandlerResponse::Error(
        //                         "Timeout waiting for MPV socket".to_string(),
        //                     );
        //                 }

        //                 let mut mpv_sock = match mpv_sock {
        //                     Ok(mpv_sock) => mpv_sock,
        //                     Err(e) => {
        //                         error!("Error connecting to mpv socket: {:?}", e);
        //                         return e;
        //                     }
        //                 };

        //                 let command_result =
        //                     send_command(&mut mpv_sock, "set_property", "pause", "no")
        //                         .map_err(|e| e.to_string())
        //                         .map_err(|e| MpvHandlerResponse::Error(e));
        //                 state.mpv_process = Some(mpv_process);
        //                 state.mpv_sock = Some(mpv_sock);
        //                 match command_result {
        //                     Ok(_) => {}
        //                     Err(e) => return e,
        //                 }
        //                 return MpvHandlerResponse::Ok;
        //             }

        //             return MpvHandlerResponse::Error("Queue is empty".to_string());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Pause(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_sock) = state.mpv_sock.borrow_mut() {
        //                 match send_command(mpv_sock, "set_property", "pause", "yes") {
        //                     Ok(_) => {}
        //                     Err(e) => return MpvHandlerResponse::Error(e),
        //                 }
        //                 return MpvHandlerResponse::Ok;
        //             }

        //             return MpvHandlerResponse::Error("No song is playing".to_string());
        //         }

        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Stop(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_process) = &mut state.mpv_process {
        //                 let kill_result =
        //                     mpv_process.kill().map_err(|e| e.to_string()).map_err(|e| {
        //                         return MpvHandlerResponse::Error(e);
        //                     });
        //                 match kill_result {
        //                     Ok(_) => {}
        //                     Err(e) => return e,
        //                 }
        //                 state.mpv_process = None;
        //                 state.mpv_sock = None;
        //                 //if there's a song in the queue, remove the first song
        //                 if !state.queue.is_empty() {
        //                     state.queue.remove(0);
        //                 }
        //                 return MpvHandlerResponse::Ok;
        //             }
        //             return MpvHandlerResponse::Error("No song is playing".to_string());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Next(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(_) = &state.mpv_sock {
        //                 //stop then play the next song
        //                 let response = self.handle_message(MpvHandlerMessage::Stop(id.clone()));
        //                 if let MpvHandlerResponse::Ok = response {
        //                     let response = self.handle_message(MpvHandlerMessage::Play(id));
        //                     return response;
        //                 }
        //                 return response;
        //             }
        //             return MpvHandlerResponse::Error("No song is playing".to_string());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Seek(id, time) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_sock) = state.mpv_sock.borrow_mut() {
        //                 match send_command_single_arg(mpv_sock, "seek", time.to_string().as_str()) {
        //                     Ok(_) => {}
        //                     Err(e) => return MpvHandlerResponse::Error(e),
        //                 }
        //                 return MpvHandlerResponse::Ok;
        //             }
        //             return MpvHandlerResponse::Error("No song is playing".to_string());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Volume(id, volume) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             let controller = self.pulse_controller.borrow_mut();
        //             let devices = controller
        //                 .list_devices()
        //                 .map_err(|e| e.to_string())
        //                 .map_err(|e| MpvHandlerResponse::Error(e));
        //             let devices = match devices {
        //                 Ok(devices) => devices,
        //                 Err(e) => return e,
        //             };
        //             let device = devices.iter().find(|d| d.name == Some(id.clone()));

        //             if let Some(dev) = device {
        //                 let pulse_volume =
        //                     Volume(((0x10000 as f64) * volume / 100.0).floor() as u32);
        //                 controller.set_device_volume_by_index(
        //                     dev.index,
        //                     &ChannelVolumes::default().set(2, pulse_volume),
        //                 );
        //                 state.volume = volume;
        //                 return MpvHandlerResponse::Ok;
        //             } else {
        //                 return MpvHandlerResponse::Error(
        //                     "Device does not exist (setting volume)".to_string(),
        //                 );
        //             }
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::GetVolume(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             return MpvHandlerResponse::Volume(state.volume);
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Add(id, song) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             state.queue.push(song);
        //             //if the queue is now only one song, play it
        //             if state.queue.len() == 1 {
        //                 let response = self.handle_message(MpvHandlerMessage::Play(id));
        //                 return response;
        //             }
        //             return MpvHandlerResponse::Ok;
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Remove(id, index) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             //removing index 0 is equivalent to stopping the song
        //             if index == 0 {
        //                 let response = self.handle_message(MpvHandlerMessage::Stop(id));
        //                 return response;
        //             }
        //             if index < state.queue.len() {
        //                 state.queue.remove(index);
        //                 return MpvHandlerResponse::Ok;
        //             }
        //             return MpvHandlerResponse::Error("Index out of bounds".to_string());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Clear(id) => {
        //         if let None = self.state.get(&id) {
        //             return MpvHandlerResponse::Error("Device does not exist".to_string());
        //         }
        //         let response = self.handle_message(MpvHandlerMessage::Stop(id.clone()));
        //         if let MpvHandlerResponse::Ok = response {
        //             if let Some(state) = self.state.get_mut(&id) {
        //                 state.queue.clear();
        //             } else {
        //                 return MpvHandlerResponse::Error(
        //                     "Device does not exist clear 2nd half".to_string(),
        //                 );
        //             }
        //             return MpvHandlerResponse::Ok;
        //         }
        //         return response;
        //     }
        //     MpvHandlerMessage::List(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             return MpvHandlerResponse::List(state.queue.clone());
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Time(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_sock) = state.mpv_sock.borrow_mut() {
        //                 let time = get_property(mpv_sock, "time-pos")
        //                     .map_err(|e| e.to_string())
        //                     .map_err(|e| MpvHandlerResponse::Error(e));
        //                 let time = match time {
        //                     Ok(time) => time,
        //                     Err(e) => return e,
        //                 };

        //                 return MpvHandlerResponse::Time(time.as_f64().unwrap_or(0.0));
        //             }
        //             return MpvHandlerResponse::Time(0.0);
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        //     MpvHandlerMessage::Duration(id) => {
        //         if let Some(state) = self.state.get_mut(&id) {
        //             if let Some(mpv_sock) = state.mpv_sock.borrow_mut() {
        //                 let duration = get_property(mpv_sock, "duration")
        //                     .map_err(|e| e.to_string())
        //                     .map_err(|e| MpvHandlerResponse::Error(e));
        //                 let duration = match duration {
        //                     Ok(duration) => duration,
        //                     Err(e) => return e,
        //                 };
        //                 return MpvHandlerResponse::Duration(duration.as_f64().unwrap_or(0.0));
        //             }
        //             return MpvHandlerResponse::Duration(0.0);
        //         }
        //         return MpvHandlerResponse::Error("Device does not exist".to_string());
        //     }
        // }
    }
}
