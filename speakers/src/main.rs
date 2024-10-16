extern crate flatbuffers;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_playlists_generated.rs"]
mod msg_playlists_generated;

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

pub mod fbs;
pub mod mpv_handler;
pub mod mpv_process;

use std::sync::Arc;

use fbs::NcSendable;
use mpv_handler::MpvHandler;
pub use msg_echo_generated::*;
pub use msg_error_generated::*;
pub use msg_playlists_generated::*;
pub use msg_print_generated::*;
pub use msg_speakers_generated::*;
pub use root_generated::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nc = Arc::new(nats::connect("nats://nats-server:4222")?);
    let mut mpv_handler = MpvHandler::new(nc.clone())?;
    //listen for messages on all subjects
    let sub = nc.subscribe("speaker.*")?;

    for msg in sub.messages() {
        let message = root_as_message(msg.data.as_slice())?;

        match message.content_type() {
            MessageContent::SpeakerCommand => {
                let content = message.content_as_speaker_command().unwrap();
                mpv_handler
                    .handle_speaker_command(content)
                    .send(&nc, "speaker.event");
            }
            MessageContent::SpeakerQuery => {
                let content = message.content_as_speaker_query().unwrap();
                mpv_handler
                    .handle_speaker_query(content)
                    .send(&nc, "speaker.event");
            }
            MessageContent::SpeakerListQuery => {
                let _ = message.content_as_speaker_list_query().unwrap();
                mpv_handler
                    .handle_speaker_list_query()
                    .send(&nc, "speaker.event");
            }
            _ => {}
        }
    }
    Ok(())
}
