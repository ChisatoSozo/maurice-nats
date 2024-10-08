use nats::Connection;

use crate::{
    Error, ErrorArgs, Message, MessageArgs, MessageContent, MusicVolumeChanged,
    MusicVolumeChangedArgs, SpeakerEvent, SpeakerEventArgs, SpeakerEventContent, SpeakerListEvent,
    SpeakerListEventArgs,
};

pub trait NcSendable {
    fn send(self, nc: &Connection, topic: &str);
}

impl NcSendable for Result<Option<Vec<u8>>, String> {
    fn send(self, nc: &Connection, topic: &str) {
        match self {
            Ok(Some(event)) => {
                nc.publish(topic, event).unwrap();
            }
            Ok(None) => {}
            Err(err) => {
                let error_message = construct_error_message(err);
                nc.publish("error", error_message).unwrap();
            }
        }
    }
}

pub fn construct_volume_changed_event_message(volume: f32, device_id: &str) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let volume_changed_event =
        MusicVolumeChanged::create(&mut fbb, &MusicVolumeChangedArgs { volume: volume });

    let device_id_str = fbb.create_string(device_id);

    let speaker_event = SpeakerEvent::create(
        &mut fbb,
        &SpeakerEventArgs {
            event_type: SpeakerEventContent::MusicVolumeChanged,
            event: Some(volume_changed_event.as_union_value()),
            device_id: Some(device_id_str),
        },
    );

    //get current time in seconds
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::SpeakerEvent,
            content: Some(speaker_event.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}

pub fn construct_speaker_list_event_message(speakers: Vec<String>) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();

    let mut speaker_list = Vec::new();
    for speaker in speakers {
        let speaker_str = fbb.create_string(&speaker);
        speaker_list.push(speaker_str);
    }

    let speaker_vec = fbb.create_vector(&speaker_list);

    let speaker_list_event = SpeakerListEvent::create(
        &mut fbb,
        &SpeakerListEventArgs {
            device_ids: Some(speaker_vec),
        },
    );

    //get current time in seconds
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::SpeakerListEvent,
            content: Some(speaker_list_event.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}

pub fn construct_error_message(error: String) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let error_str = fbb.create_string(&error);
    let from_str = fbb.create_string("speakers");

    let error = Error::create(
        &mut fbb,
        &ErrorArgs {
            from: Some(from_str),
            message: Some(error_str),
        },
    );

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp: 0,
            content_type: MessageContent::Error,
            content: Some(error.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}
