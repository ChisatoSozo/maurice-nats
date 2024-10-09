extern crate flatbuffers;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_speakers_generated.rs"]
mod msg_speakers_generated;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_echo_generated.rs"]
mod msg_echo_generated;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_print_generated.rs"]
mod msg_print_generated;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_error_generated.rs"]
mod msg_error_generated;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/root_generated.rs"]
mod root_generated;

pub use msg_echo_generated::*;
pub use msg_error_generated::*;
pub use msg_print_generated::*;
pub use msg_speakers_generated::*;
pub use root_generated::*;

use std::fs::OpenOptions;
use std::io::Write;

const NATS_ECHO_SUBJECT: &str = "echo";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nc = nats::connect("nats://nats-server:4222")?;
    let sub = nc.subscribe(">")?;

    for msg in sub.messages() {
        // Skip echo messages to avoid recursion
        if msg.subject == NATS_ECHO_SUBJECT {
            continue;
        }

        // Deserialize the received message
        let message = root_as_message(msg.data.as_slice())?;
        let timestamp = message.timestamp();

        // Process the message to get the formatted string
        let message_content_stringified = process_message(&message);

        // Build the full string with subject, type, and content
        let message_type_stringified = format!("{:?}", message.content_type());
        let full_string = format!(
            "[{}] {}: {}",
            msg.subject, message_type_stringified, message_content_stringified
        );

        // Construct and send an echo message
        send_echo_message(&nc, timestamp, full_string.clone())?;

        // Log the message
        println!("{}", full_string);

        // Write to /dev/ttyhost
        if let Err(e) = write_to_ttyhost(&full_string) {
            eprintln!("Failed to write to /dev/ttyhost: {}", e);
        }
    }
    Ok(())
}

fn process_message(message: &Message) -> String {
    let message_type = message.content_type();
    const MAX_MESSAGE_CONTENT: u8 = MessageContent::ENUM_MAX as u8 + 1;

    match message_type {
        MessageContent::NONE => "NONE".to_string(),

        MessageContent::Print => {
            if let Some(print_message) = message.content_as_print() {
                format_message_content("Print", print_message.message())
            } else {
                error_message("Print", "content_as_print was None")
            }
        }

        MessageContent::Echo => {
            if let Some(echo_message) = message.content_as_echo() {
                format_message_content("Echo", echo_message.message())
            } else {
                error_message("Echo", "content_as_echo was None")
            }
        }

        MessageContent::Error => {
            if let Some(error_message) = message.content_as_error() {
                format_message_content("Error", error_message.message())
            } else {
                error_message("Error", "content_as_error was None")
            }
        }

        MessageContent::SpeakerCommand => handle_speaker_command(message),
        MessageContent::SpeakerQuery => handle_speaker_query(message),
        MessageContent::SpeakerEvent => handle_speaker_event(message),

        MessageContent::SpeakerListQuery => "SpeakerListQuery".to_string(),

        MessageContent::SpeakerListEvent => {
            if let Some(speaker_list_event) = message.content_as_speaker_list_event() {
                if let Some(device_ids) = speaker_list_event.device_ids() {
                    let ids: Vec<&str> = device_ids.iter().collect();
                    format!("SpeakerListEvent: device_ids={:?}", ids)
                } else {
                    error_message("SpeakerListEvent", "device_ids was None")
                }
            } else {
                error_message("SpeakerListEvent", "content_as_speaker_list_event was None")
            }
        }

        // Ensure exhaustive matching
        MessageContent(MAX_MESSAGE_CONTENT..=u8::MAX) => "UNKNOWN MessageContent".to_string(),
    }
}

fn format_message_content(content_type: &str, message: Option<&str>) -> String {
    let msg = message.unwrap_or("{NO MESSAGE}");
    format!("{}: {}", content_type, msg)
}

fn error_message(content_type: &str, error: &str) -> String {
    format!("{{ERROR: TYPE WAS {} BUT {}}}", content_type, error)
}

fn handle_speaker_command(message: &Message) -> String {
    if let Some(speaker_command) = message.content_as_speaker_command() {
        const MAX_SPEAKER_COMMAND_CONTENT: u8 = SpeakerCommandContent::ENUM_MAX as u8 + 1;

        match speaker_command.command_type() {
            SpeakerCommandContent::NONE => "SpeakerCommand: NONE".to_string(),

            SpeakerCommandContent::SetMusicVolume => {
                if let Some(set_music_volume) = speaker_command.command_as_set_music_volume() {
                    format!("SetMusicVolume: volume={}", set_music_volume.volume())
                } else {
                    error_message("SetMusicVolume", "command_as_set_music_volume was None")
                }
            }

            SpeakerCommandContent::TogglePause => "TogglePause".to_string(),

            SpeakerCommandContent::Play => handle_play_command(&speaker_command),

            SpeakerCommandContent::Stop => "Stop".to_string(),

            SpeakerCommandContent::Seek => {
                if let Some(seek) = speaker_command.command_as_seek() {
                    format!("Seek: seek={}", seek.seek())
                } else {
                    error_message("Seek", "command_as_seek was None")
                }
            }

            // Ensure exhaustive matching
            SpeakerCommandContent(MAX_SPEAKER_COMMAND_CONTENT..=u8::MAX) => {
                "UNKNOWN SpeakerCommandContent".to_string()
            }
        }
    } else {
        error_message("SpeakerCommand", "content_as_speaker_command was None")
    }
}

fn handle_play_command(speaker_command: &SpeakerCommand) -> String {
    if let Some(play) = speaker_command.command_as_play() {
        const MAX_PLAY_CONTENT: u8 = PlayContent::ENUM_MAX as u8 + 1;

        match play.content_type() {
            PlayContent::NONE => "PlayContent: NONE".to_string(),

            PlayContent::PlayYoutube => {
                if let Some(play_youtube) = play.content_as_play_youtube() {
                    let url = play_youtube.url().unwrap_or("{NO URL}");
                    format!("PlayYoutube: url={}", url)
                } else {
                    error_message("PlayYoutube", "content_as_play_youtube was None")
                }
            }

            PlayContent::PlayWav => {
                // We can't print raw WAV data
                "PlayWav: [RAW WAV DATA]".to_string()
            }

            // Ensure exhaustive matching
            PlayContent(MAX_PLAY_CONTENT..=u8::MAX) => "UNKNOWN PlayContent".to_string(),
        }
    } else {
        error_message("Play", "command_as_play was None")
    }
}

fn handle_speaker_query(message: &Message) -> String {
    if let Some(speaker_query) = message.content_as_speaker_query() {
        const MAX_SPEAKER_QUERY_CONTENT: u8 = SpeakerQueryContent::ENUM_MAX as u8 + 1;

        match speaker_query.query_type() {
            SpeakerQueryContent::NONE => "SpeakerQuery: NONE".to_string(),

            SpeakerQueryContent::QueryMusicVolume => "QueryMusicVolume".to_string(),

            SpeakerQueryContent::QueryPause => "QueryPause".to_string(),

            SpeakerQueryContent::QueryPlay => "QueryPlay".to_string(),

            SpeakerQueryContent::QuerySeek => "QuerySeek".to_string(),

            SpeakerQueryContent::QueryDuration => "QueryDuration".to_string(),

            // Ensure exhaustive matching
            SpeakerQueryContent(MAX_SPEAKER_QUERY_CONTENT..=u8::MAX) => {
                "UNKNOWN SpeakerQueryContent".to_string()
            }
        }
    } else {
        error_message("SpeakerQuery", "content_as_speaker_query was None")
    }
}

fn handle_speaker_event(message: &Message) -> String {
    if let Some(speaker_event) = message.content_as_speaker_event() {
        const MAX_SPEAKER_EVENT_CONTENT: u8 = SpeakerEventContent::ENUM_MAX as u8 + 1;

        match speaker_event.event_type() {
            SpeakerEventContent::NONE => "SpeakerEvent: NONE".to_string(),

            SpeakerEventContent::MusicVolumeChanged => {
                if let Some(music_volume_changed) = speaker_event.event_as_music_volume_changed() {
                    format!(
                        "MusicVolumeChanged: volume={}",
                        music_volume_changed.volume()
                    )
                } else {
                    error_message(
                        "MusicVolumeChanged",
                        "event_as_music_volume_changed was None",
                    )
                }
            }

            SpeakerEventContent::PlayStarted => "PlayStarted".to_string(),

            SpeakerEventContent::PlayStopped => "PlayStopped".to_string(),

            SpeakerEventContent::SeekChanged => {
                if let Some(seek_changed) = speaker_event.event_as_seek_changed() {
                    format!("SeekChanged: seek={}", seek_changed.seek())
                } else {
                    error_message("SeekChanged", "event_as_seek_changed was None")
                }
            }

            SpeakerEventContent::DurationChanged => {
                if let Some(duration_changed) = speaker_event.event_as_duration_changed() {
                    format!("DurationChanged: duration={}", duration_changed.duration())
                } else {
                    error_message("DurationChanged", "event_as_duration_changed was None")
                }
            }

            SpeakerEventContent::FileEnded => "FileEnded".to_string(),
            SpeakerEventContent::PauseChanged => {
                if let Some(pause_changed) = speaker_event.event_as_pause_changed() {
                    format!("PauseChanged: paused={}", pause_changed.paused())
                } else {
                    error_message("PauseChanged", "event_as_pause_changed was None")
                }
            }

            // Ensure exhaustive matching
            SpeakerEventContent(MAX_SPEAKER_EVENT_CONTENT..=u8::MAX) => {
                "UNKNOWN SpeakerEventContent".to_string()
            }
        }
    } else {
        error_message("SpeakerEvent", "content_as_speaker_event was None")
    }
}

fn send_echo_message(
    nc: &nats::Connection,
    timestamp: u64,
    full_string: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let message_string = fbb.create_string(&full_string);
    let echo_message = Echo::create(
        &mut fbb,
        &EchoArgs {
            message: Some(message_string),
        },
    );
    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::Echo,
            content: Some(echo_message.as_union_value()),
        },
    );
    fbb.finish(root, None);

    nc.publish(NATS_ECHO_SUBJECT, fbb.finished_data())?;
    Ok(())
}

fn write_to_ttyhost(full_string: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).open("/dev/ttyhost")?;
    writeln!(file, "{}", full_string)
}
