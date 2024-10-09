use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use crate::{
    fbs::{
        construct_duration_changed_event_message, construct_file_ended_event_message,
        construct_play_stopped_event_message, construct_seek_changed_event_message,
        construct_speaker_list_event_message, construct_volume_changed_event_message,
    },
    PlayContent, SpeakerCommand, SpeakerCommandContent, SpeakerListQuery, SpeakerQuery,
    SpeakerQueryContent,
};

use alsa::{device_name::HintIter, Direction};
use serde_json::{json, Value};

enum MpvEvent {
    Seek {
        device_id: String,
        playback_time: f32,
    },
    EndFile {
        device_id: String,
    },
}

enum MpvCommand {
    Send {
        request_id: i32,
        command: Value,
        response_tx: Sender<Value>,
    },
}

struct SpeakerState {
    music_volume: f32,
    mpv_process: Option<std::process::Child>,
    command_sender: Option<Sender<MpvCommand>>,
    next_request_id: i32,
}

pub struct MpvHandler<'a> {
    state: HashMap<String, SpeakerState>,
    event_receiver: Receiver<MpvEvent>,
    event_sender: Sender<MpvEvent>,
    nc: &'a nats::Connection,
}

pub fn list_pcm_devices(direction: Direction) -> Vec<String> {
    let hints = HintIter::new_str(None, "pcm").unwrap();
    let mut devices = Vec::new();
    for hint in hints {
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

impl<'a> MpvHandler<'a> {
    pub fn new(nc: &'a nats::Connection) -> Result<Self, String> {
        let (event_sender, event_receiver) = mpsc::channel();
        Ok(Self {
            state: HashMap::new(),
            event_receiver,
            event_sender,
            nc,
        })
    }

    pub fn handle_speaker_list_query(
        &mut self,
        _: SpeakerListQuery,
    ) -> Result<Option<Vec<u8>>, String> {
        let playback_devices = list_pcm_devices(Direction::Playback);

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
                        command_sender: None,
                        next_request_id: 1,
                    },
                );
            }
        }

        Ok(Some(construct_speaker_list_event_message(playback_devices)))
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

        match command.command_type() {
            SpeakerCommandContent::SetMusicVolume => {
                let volume = command
                    .command_as_set_music_volume()
                    .ok_or("No volume")?
                    .volume();

                // Log the volume change
                println!("Setting volume to {} for device {}", volume, device_id);

                // Update the stored volume
                speaker_state.music_volume = volume;

                // Send command and wait for response
                send_command_and_wait(
                    speaker_state,
                    vec!["set_property", "volume", &volume.to_string()],
                )?;

                Ok(Some(construct_volume_changed_event_message(
                    volume, device_id,
                )))
            }
            SpeakerCommandContent::TogglePause => {
                println!("Toggling pause for device {}", device_id);
                send_command_and_wait(speaker_state, vec!["cycle", "pause"])?;
                Ok(None)
            }
            SpeakerCommandContent::Play => {
                if speaker_state.mpv_process.is_some() {
                    // Terminate the current mpv process
                    let mpv_process = speaker_state.mpv_process.as_mut().ok_or("No mpv process")?;
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    speaker_state.mpv_process = None;
                    speaker_state.command_sender = None;
                }

                let play_command = command.command_as_play().ok_or("No play command")?;

                match play_command.content_type() {
                    PlayContent::PlayYoutube => {
                        let url = play_command
                            .content_as_play_youtube()
                            .ok_or("No YouTube URL")?
                            .url()
                            .ok_or("URL is None")?;
                        let socket_path = format!("/tmp/mpv-socket-{}", device_id);

                        println!("Starting mpv for URL: {} on device {}", url, device_id);

                        let mpv_process = std::process::Command::new("mpv")
                            .arg(url)
                            .arg("--no-video")
                            .arg(format!("--audio-device=alsa/{device_id}"))
                            .arg(format!("--input-ipc-server={}", socket_path))
                            .spawn()
                            .map_err(|e| e.to_string())?;

                        // Wait for the socket to become available
                        let timeout = Duration::from_secs(5);
                        let start_time = Instant::now();

                        while start_time.elapsed() < timeout {
                            if UnixStream::connect(&socket_path).is_ok() {
                                println!("MPV socket connected for device {}", device_id);
                                break;
                            }
                            thread::sleep(Duration::from_millis(100));
                        }

                        // Create channels for communication
                        let (command_sender, command_receiver) = mpsc::channel();

                        // Spawn the MPV handler thread
                        let event_sender = self.event_sender.clone();
                        let device_id_clone = device_id.to_string();

                        thread::spawn(move || {
                            if let Err(e) = mpv_handler_thread(
                                socket_path,
                                command_receiver,
                                event_sender,
                                device_id_clone,
                            ) {
                                eprintln!("MPV handler thread error: {}", e);
                            }
                        });

                        speaker_state.mpv_process = Some(mpv_process);
                        speaker_state.command_sender = Some(command_sender);
                        speaker_state.next_request_id = 1;

                        // **Wait until MPV is ready to receive commands**
                        let timeout = Duration::from_secs(5);
                        let start_time = Instant::now();

                        loop {
                            match send_command_and_wait(
                                speaker_state,
                                vec!["get_property", "playback-time"],
                            ) {
                                Ok(_) => {
                                    println!("MPV is ready to receive commands");
                                    break;
                                }
                                Err(e) => {
                                    if start_time.elapsed() >= timeout {
                                        return Err(format!(
                                            "MPV did not become ready in time: {}",
                                            e
                                        ));
                                    }
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }

                        // Set volume to the stored music_volume
                        send_command_and_wait(
                            speaker_state,
                            vec![
                                "set_property",
                                "volume",
                                &speaker_state.music_volume.to_string(),
                            ],
                        )?;

                        // No longer observing properties here

                        Ok(None)
                    }
                    _ => Err("Unsupported play content".to_string()),
                }
            }
            SpeakerCommandContent::Stop => {
                if let Some(mpv_process) = &mut speaker_state.mpv_process {
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    speaker_state.mpv_process = None;
                    speaker_state.command_sender = None;

                    // Send PlayStopped event
                    let event_message = construct_play_stopped_event_message(device_id);
                    self.nc.publish("speaker.event", event_message).unwrap();
                }
                Ok(None)
            }
            SpeakerCommandContent::Seek => {
                let seek_command = command.command_as_seek().ok_or("No seek command")?;
                let seek_value = seek_command.seek();

                send_command_and_wait(
                    speaker_state,
                    vec!["set_property", "playback-time", &seek_value.to_string()],
                )?;

                Ok(Some(construct_seek_changed_event_message(
                    seek_value, device_id,
                )))
            }
            _ => Err("Unknown command".to_string()),
        }
    }

    pub fn handle_speaker_query(&mut self, query: SpeakerQuery) -> Result<Option<Vec<u8>>, String> {
        let device_id: &str = query.device_id().ok_or("No device_id")?;
        let speaker_state = self
            .state
            .get_mut(device_id)
            .ok_or("Device does not exist")?;

        match query.query_type() {
            SpeakerQueryContent::QueryMusicVolume => {
                let volume = speaker_state.music_volume;
                Ok(Some(construct_volume_changed_event_message(
                    volume, device_id,
                )))
            }
            SpeakerQueryContent::QuerySeek => {
                let seek = get_property(speaker_state, "playback-time")?
                    .as_f64()
                    .unwrap_or(0.0) as f32;
                Ok(Some(construct_seek_changed_event_message(seek, device_id)))
            }
            SpeakerQueryContent::QueryDuration => {
                let duration = get_property(speaker_state, "duration")?
                    .as_f64()
                    .unwrap_or(0.0) as f32;
                Ok(Some(construct_duration_changed_event_message(
                    duration, device_id,
                )))
            }
            _ => Err("Unknown query".to_string()),
        }
    }

    pub fn process_events(&mut self) -> Result<(), String> {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                MpvEvent::Seek {
                    device_id,
                    playback_time,
                } => {
                    let event_message =
                        construct_seek_changed_event_message(playback_time, &device_id);
                    self.nc.publish("speaker.event", event_message).unwrap();
                }
                MpvEvent::EndFile { device_id } => {
                    let event_message = construct_file_ended_event_message(&device_id);
                    self.nc.publish("speaker.event", event_message).unwrap();
                }
            }
        }
        Ok(())
    }
}

fn send_command_and_wait(
    speaker_state: &mut SpeakerState,
    args: Vec<&str>,
) -> Result<Value, String> {
    let request_id = speaker_state.next_request_id;
    speaker_state.next_request_id += 1;

    let (response_tx, response_rx) = mpsc::channel();
    let args_str = args.join(",");
    let mut command_array = Vec::new();
    for arg in args {
        command_array.push(json!(arg));
    }

    let command_json = json!({
        "command": command_array,
        "request_id": request_id
    });

    let command_json_str = command_json.to_string();

    let command_sender = speaker_state
        .command_sender
        .as_ref()
        .ok_or("MPV command sender not available")?;

    command_sender
        .send(MpvCommand::Send {
            request_id,
            command: command_json,
            response_tx,
        })
        .map_err(|e| e.to_string())?;

    match response_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(response) => {
            if response["error"] == json!("success") {
                Ok(response)
            } else {
                Err(format!(
                    "mpv returned error, args were: [{}], error is: {}, \nCOMMAND:\n{}",
                    args_str,
                    response["error"].as_str().unwrap_or("Unknown error"),
                    command_json_str
                ))
            }
        }
        Err(e) => Err(format!("Failed to receive response: {}", e)),
    }
}

fn get_property(speaker_state: &mut SpeakerState, property: &str) -> Result<Value, String> {
    send_command_and_wait(speaker_state, vec!["get_property", property]).and_then(|response| {
        if response["error"] == json!("success") {
            Ok(response["data"].clone())
        } else {
            Err(format!(
                "Error getting property '{}': {}",
                property, response["error"]
            ))
        }
    })
}

fn mpv_handler_thread(
    socket_path: String,
    command_receiver: Receiver<MpvCommand>,
    event_sender: Sender<MpvEvent>,
    device_id: String,
) -> Result<(), String> {
    let mut mpv_sock = UnixStream::connect(&socket_path).map_err(|e| e.to_string())?;
    mpv_sock.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mpv_sock_reader = mpv_sock.try_clone().map_err(|e| e.to_string())?;
    let reader = BufReader::new(mpv_sock_reader);

    let mut pending_requests: HashMap<i32, Sender<Value>> = HashMap::new();
    let mut lines = reader.lines();
    let mut next_request_id = 1000;

    loop {
        // Handle incoming commands
        while let Ok(cmd) = command_receiver.try_recv() {
            match cmd {
                MpvCommand::Send {
                    request_id,
                    command,
                    response_tx,
                } => {
                    pending_requests.insert(request_id, response_tx);
                    writeln!(mpv_sock, "{}", command.to_string()).map_err(|e| e.to_string())?;
                    mpv_sock.flush().map_err(|e| e.to_string())?;
                }
            }
        }

        // Handle MPV responses and events
        match lines.next() {
            Some(Ok(line)) => {
                let event: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error parsing JSON from mpv: {}", e);
                        continue;
                    }
                };

                // Check if this is a response to a command
                if let Some(request_id) = event.get("request_id").and_then(|v| v.as_i64()) {
                    let request_id = request_id as i32;
                    if let Some(sender) = pending_requests.remove(&request_id) {
                        sender.send(event.clone()).unwrap_or_else(|e| {
                            eprintln!("Failed to send response for request {}: {}", request_id, e);
                        });
                        continue;
                    }
                }

                // Handle events
                if let Some(event_name) = event.get("event").and_then(|e| e.as_str()) {
                    match event_name {
                        "seek" => {
                            // Handle seek event
                            let playback_time = match get_property_value(
                                &mut mpv_sock,
                                &mut pending_requests,
                                "playback-time",
                                &mut next_request_id,
                            ) {
                                Ok(value) => value.as_f64().unwrap_or(0.0) as f32,
                                Err(e) => {
                                    eprintln!("Error getting playback-time: {}", e);
                                    continue;
                                }
                            };
                            event_sender
                                .send(MpvEvent::Seek {
                                    device_id: device_id.clone(),
                                    playback_time,
                                })
                                .unwrap_or_else(|e| {
                                    eprintln!("Failed to send seek event: {}", e);
                                });
                        }
                        "end-file" => {
                            event_sender
                                .send(MpvEvent::EndFile {
                                    device_id: device_id.clone(),
                                })
                                .unwrap_or_else(|e| {
                                    eprintln!("Failed to send end-file event: {}", e);
                                });
                        }
                        _ => {}
                    }
                }
            }
            Some(Err(e)) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    eprintln!("Error reading from mpv socket: {}", e);
                }
            }
            None => {
                // Sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn get_property_value(
    mpv_sock: &mut UnixStream,
    pending_requests: &mut HashMap<i32, Sender<Value>>,
    property: &str,
    next_request_id: &mut i32,
) -> Result<Value, String> {
    let request_id = *next_request_id;
    *next_request_id += 1;

    let (response_tx, response_rx) = mpsc::channel();

    pending_requests.insert(request_id, response_tx);

    let command_json = json!({
        "command": ["get_property", property],
        "request_id": request_id
    });

    writeln!(mpv_sock, "{}", command_json.to_string()).map_err(|e| e.to_string())?;
    mpv_sock.flush().map_err(|e| e.to_string())?;

    match response_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(response) => {
            if response["error"] == json!("success") {
                Ok(response["data"].clone())
            } else {
                Err(format!(
                    "mpv returned error: {}",
                    response["error"].as_str().unwrap_or("Unknown error")
                ))
            }
        }
        Err(e) => Err(format!("Failed to receive response: {}", e)),
    }
}
