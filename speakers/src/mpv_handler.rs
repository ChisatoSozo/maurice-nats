use std::{collections::HashMap, sync::Arc};

use crate::{
    fbs::construct_speaker_list_event_message, mpv_process::MpvProcess, PlayContent,
    SpeakerCommand, SpeakerCommandContent, SpeakerQuery, SpeakerQueryContent,
};

use alsa::{device_name::HintIter, Direction};

struct SpeakerState {
    music_volume: f32,
    mpv_process: Option<MpvProcess>,
}

pub struct MpvHandler {
    state: HashMap<String, SpeakerState>,
    nc: Arc<nats::Connection>,
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

impl MpvHandler {
    pub fn new(nc: Arc<nats::Connection>) -> Result<Self, String> {
        Ok(Self {
            state: HashMap::new(),
            nc,
        })
    }

    pub fn handle_speaker_list_query(&mut self) -> Result<Option<Vec<u8>>, String> {
        let playback_devices = list_pcm_devices(Direction::Playback);

        let mut devices_to_remove = Vec::new();
        for (device_id, _) in self.state.iter() {
            if !playback_devices.contains(&device_id) {
                devices_to_remove.push(device_id.clone());
            }
        }

        for device_id in &playback_devices {
            if !self.state.contains_key(device_id) {
                self.state.insert(
                    device_id.clone(),
                    SpeakerState {
                        music_volume: 100.0,
                        mpv_process: None,
                    },
                );
                // No need to start mpv_handler_thread here as it's already started in new()
            }
        }

        Ok(Some(construct_speaker_list_event_message(playback_devices)))
    }

    pub fn handle_speaker_command(
        &mut self,
        command: SpeakerCommand,
    ) -> Result<Option<Vec<u8>>, String> {
        self.handle_speaker_list_query()?;

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
                speaker_state.music_volume = volume;

                if let Some(mpv_process) = &mut speaker_state.mpv_process {
                    mpv_process
                        .set_property("volume", volume)
                        .map_err(|e| e.to_string())?;
                    mpv_process
                        .query_property("volume")
                        .map_err(|e| e.to_string())?;
                }

                Ok(None)
            }
            SpeakerCommandContent::TogglePause => {
                println!("Toggling pause for device {}", device_id);

                Ok(None)
            }
            SpeakerCommandContent::Play => {
                if speaker_state.mpv_process.is_some() {
                    // Terminate the current mpv process
                    let mpv_process = speaker_state.mpv_process.as_mut().ok_or("No mpv process")?;
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    speaker_state.mpv_process = None;
                }

                let play_command = command.command_as_play().ok_or("No play command")?;

                match play_command.content_type() {
                    PlayContent::PlayYoutube => {
                        let url = play_command
                            .content_as_play_youtube()
                            .ok_or("No YouTube URL")?
                            .url()
                            .ok_or("URL is None")?;

                        speaker_state.mpv_process = Some(
                            MpvProcess::new(url, device_id, self.nc.clone())
                                .map_err(|e| e.to_string())?,
                        );

                        speaker_state
                            .mpv_process
                            .as_mut()
                            .unwrap()
                            .set_property("volume", speaker_state.music_volume)
                            .map_err(|e| e.to_string())?;

                        Ok(None)
                    }
                    _ => Err("Unsupported play content".to_string()),
                }
            }
            SpeakerCommandContent::Stop => {
                if let Some(mpv_process) = &mut speaker_state.mpv_process {
                    mpv_process.kill().map_err(|e| e.to_string())?;
                    speaker_state.mpv_process = None;
                }
                Ok(None)
            }
            SpeakerCommandContent::Seek => {
                let seek_command = command.command_as_seek().ok_or("No seek command")?;
                let seek_time = seek_command.seek();

                if let Some(mpv_process) = &mut speaker_state.mpv_process {
                    mpv_process
                        .set_property("time-pos", seek_time)
                        .map_err(|e| e.to_string())?;
                    mpv_process
                        .query_property("time-pos")
                        .map_err(|e| e.to_string())?;
                }

                Ok(None)
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

        let mpv_process = speaker_state.mpv_process.as_mut().ok_or("No mpv process")?;

        match query.query_type() {
            SpeakerQueryContent::QueryMusicVolume => {
                mpv_process
                    .query_property("volume")
                    .map_err(|e| e.to_string())?;
                Ok(None)
            }
            SpeakerQueryContent::QuerySeek => {
                mpv_process
                    .query_property("time-pos")
                    .map_err(|e| e.to_string())?;
                Ok(None)
            }
            SpeakerQueryContent::QueryDuration => {
                mpv_process
                    .query_property("duration")
                    .map_err(|e| e.to_string())?;
                Ok(None)
            }
            _ => Err("Unknown query".to_string()),
        }
    }
}
