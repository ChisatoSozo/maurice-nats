use nats::Connection;

use crate::{
    Error, ErrorArgs, FileEnded, FileEndedArgs, Message, MessageArgs, MessageContent,
    MusicVolumeChanged, MusicVolumeChangedArgs, PauseChanged, PauseChangedArgs, Play, PlayArgs,
    PlayContent, PlayStopped, PlayStoppedArgs, PlayYoutube, PlayYoutubeArgs, PlaylistEvent,
    PlaylistEventArgs, PlaylistEventContent, PlaylistStateChanged, PlaylistStateChangedArgs,
    SeekChanged, SeekChangedArgs, Song, SongArgs, SongInternal, SpeakerCommand, SpeakerCommandArgs,
    SpeakerCommandContent, SpeakerEvent, SpeakerEventArgs, SpeakerEventContent, SpeakerListEvent,
    SpeakerListEventArgs, Stop, StopArgs,
};

pub trait NcSendable {
    fn send(self, nc: &Connection, topic: &str, from: &str);
}

impl NcSendable for Result<Option<Vec<u8>>, String> {
    fn send(self, nc: &Connection, topic: &str, from: &str) {
        match self {
            Ok(Some(event)) => {
                nc.publish(topic, event).unwrap();
            }
            Ok(None) => {}
            Err(err) => {
                let error_message = construct_error_message(&err.to_string(), from);
                nc.publish("error", error_message).unwrap();
            }
        }
    }
}

pub trait UnwrapNc<T> {
    fn unwrap_nc(self, nc: &Connection, from: &str) -> Option<T>;
}

impl<T, E: ToString> UnwrapNc<T> for Result<T, E> {
    fn unwrap_nc(self, nc: &Connection, from: &str) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                let error_message = construct_error_message(&err.to_string(), from);
                nc.publish("error", error_message).unwrap();
                None
            }
        }
    }
}

pub fn construct_error_message(error: &str, from: &str) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let error_str = fbb.create_string(error);
    let from_str = fbb.create_string(from);

    let error = Error::create(
        &mut fbb,
        &ErrorArgs {
            from: Some(from_str),
            message: Some(error_str),
        },
    );

    let timestamp = get_current_timestamp();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::Error,
            content: Some(error.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}

pub fn send_error_message(nc: &Connection, error: &str, from: &str) {
    let error_message = construct_error_message(error, from);
    nc.publish("error", error_message).unwrap();
}

fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn construct_play_youtube_song_command(url: String, device_id: String) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let url_str = fbb.create_string(&url);

    let play_youtube_command =
        PlayYoutube::create(&mut fbb, &PlayYoutubeArgs { url: Some(url_str) });

    let play_command = Play::create(
        &mut fbb,
        &PlayArgs {
            content_type: PlayContent::PlayYoutube,
            content: Some(play_youtube_command.as_union_value()),
        },
    );

    let device_id_str = fbb.create_string(&device_id);

    let command = SpeakerCommand::create(
        &mut fbb,
        &SpeakerCommandArgs {
            device_id: Some(device_id_str),
            command_type: SpeakerCommandContent::Play,
            command: Some(play_command.as_union_value()),
        },
    );

    let timestamp = get_current_timestamp();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::SpeakerCommand,
            content: Some(command.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}

pub fn construct_stop_command(device_id: String) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();

    let stop_command = Stop::create(&mut fbb, &StopArgs {});

    let device_id_str = fbb.create_string(&device_id);

    let command = SpeakerCommand::create(
        &mut fbb,
        &SpeakerCommandArgs {
            device_id: Some(device_id_str),
            command_type: SpeakerCommandContent::Stop,
            command: Some(stop_command.as_union_value()),
        },
    );

    let timestamp = get_current_timestamp();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::SpeakerCommand,
            content: Some(command.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}

pub fn construct_playlist_updated_event(
    playlist: &Vec<SongInternal>,
    device_id: String,
) -> Vec<u8> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let mut songs = Vec::new();
    for song in playlist {
        let url_str = fbb.create_string(&song.url);
        let title_str = fbb.create_string(&song.title);
        let thumbnail_b64_str = fbb.create_string(&song.thumbnail_b64);

        let song = Song::create(
            &mut fbb,
            &SongArgs {
                url: Some(url_str),
                title: Some(title_str),
                thumbnail_b64: Some(thumbnail_b64_str),
            },
        );
        songs.push(song);
    }

    let vec = fbb.create_vector(&songs);

    let playlist =
        PlaylistStateChanged::create(&mut fbb, &PlaylistStateChangedArgs { songs: Some(vec) });

    let device_id_str = fbb.create_string(&device_id);

    let playlist_event = PlaylistEvent::create(
        &mut fbb,
        &PlaylistEventArgs {
            device_id: Some(device_id_str),
            event_type: PlaylistEventContent::PlaylistStateChanged,
            event: Some(playlist.as_union_value()),
        },
    );

    let timestamp = get_current_timestamp();

    let root = Message::create(
        &mut fbb,
        &MessageArgs {
            timestamp,
            content_type: MessageContent::PlaylistEvent,
            content: Some(playlist_event.as_union_value()),
        },
    );
    fbb.finish(root, None);

    fbb.finished_data().to_vec()
}
