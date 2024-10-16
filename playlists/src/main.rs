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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use fbs::{
    construct_play_youtube_song_command, construct_playlist_updated_event, construct_stop_command,
    send_error_message, UnwrapNc,
};

pub use msg_echo_generated::*;
pub use msg_error_generated::*;
pub use msg_playlists_generated::*;
pub use msg_print_generated::*;
pub use msg_speakers_generated::*;
pub use root_generated::*;

pub struct SongInternal {
    url: String,
    title: String,
    thumbnail_b64: String,
}

fn new_song_at_position_0(
    nc: Arc<nats::Connection>,
    device_id: String,
    playlist: &Vec<SongInternal>,
) {
    let song = &playlist[0];
    let url = &song.url;
    nc.publish(
        "speaker.command",
        construct_play_youtube_song_command(url.to_string(), device_id.to_string()),
    )
    .unwrap_nc(&nc, "playlist");
}

fn stop_command(nc: Arc<nats::Connection>, device_id: String) {
    nc.publish("speaker.command", construct_stop_command(device_id))
        .unwrap_nc(&nc, "playlist");
}

fn playlist_updated_event(
    nc: Arc<nats::Connection>,
    device_id: String,
    playlist: &Vec<SongInternal>,
) {
    nc.publish(
        "playlist.event",
        construct_playlist_updated_event(playlist, device_id.to_string()),
    )
    .unwrap_nc(&nc, "playlist");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playlists = Arc::new(Mutex::new(HashMap::<String, Vec<SongInternal>>::new()));

    // Connect to the NATS server
    let nc = Arc::new(nats::connect("nats://nats-server:4222")?);

    // Subscribe to both "playlist.*" and "speaker.event"
    let sub_playlist = nc.subscribe("playlist.*")?;
    let sub_speaker = nc.subscribe("speaker.event")?;

    let nc_clone = nc.clone();
    let playlists_clone = playlists.clone();
    // Spawn a thread to handle playlist messages
    let playlist_thread = thread::spawn(move || {
        for msg in sub_playlist.messages() {
            let nc_clone = nc_clone.clone();
            let message = root_as_message(msg.data.as_slice()).unwrap_nc(&nc_clone, "playlist");
            match message {
                Some(message) => match message.content_type() {
                    MessageContent::PlaylistQuery => {
                        let query = message.content_as_playlist_query().unwrap();
                        let query_type = query.query_type();
                        let device_id = query
                            .device_id()
                            .ok_or("Device id is none")
                            .unwrap_nc(&nc_clone, "playlist");
                        let device_id = match device_id {
                            Some(device_id) => device_id,
                            None => continue,
                        };

                        const MAX: u8 = PlaylistQueryContent::ENUM_MAX + 1;
                        match query_type {
                            PlaylistQueryContent::NONE => {
                                send_error_message(&nc_clone, "Invalid query type", "playlist");
                            }
                            PlaylistQueryContent::QueryPlaylistState => {
                                playlists_clone.lock().unwrap_nc(&nc_clone, "playlist").map(
                                    |playlists| {
                                        let blank = Vec::new();
                                        let playlist =
                                            playlists.get(&device_id.to_string()).unwrap_or(&blank);
                                        playlist_updated_event(
                                            nc_clone,
                                            device_id.to_string(),
                                            playlist,
                                        );
                                    },
                                );
                            }
                            PlaylistQueryContent(MAX..=u8::MAX) => {
                                send_error_message(&nc_clone, "Invalid query type", "playlist");
                            }
                        }
                    }
                    MessageContent::PlaylistCommand => {
                        let command = message.content_as_playlist_command().unwrap();
                        let command_type = command.command_type();
                        let device_id = command
                            .device_id()
                            .ok_or("Device id is none")
                            .unwrap_nc(&nc_clone, "playlist");
                        let device_id = match device_id {
                            Some(device_id) => device_id,
                            None => continue,
                        };

                        const MAX: u8 = PlaylistCommandContent::ENUM_MAX + 1;
                        match command_type {
                            PlaylistCommandContent::NONE => {
                                send_error_message(&nc_clone, "Invalid command type", "playlist");
                            }
                            PlaylistCommandContent::AddSong => {
                                let command = command.command_as_add_song().unwrap();
                                let song = command
                                    .song()
                                    .ok_or("Song is none")
                                    .unwrap_nc(&nc_clone, "playlist");
                                match song {
                                    Some(song) => {
                                        let title = song
                                            .title()
                                            .ok_or("Title is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let url = song
                                            .url()
                                            .ok_or("Url is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let thumbnail_b64 = song
                                            .thumbnail_b64()
                                            .ok_or("Thumbnail is none")
                                            .unwrap_nc(&nc_clone, "playlist");

                                        match (title, url, thumbnail_b64) {
                                            (Some(title), Some(url), Some(thumbnail_b64)) => {
                                                let song_internal = SongInternal {
                                                    url: url.to_string(),
                                                    title: title.to_string(),
                                                    thumbnail_b64: thumbnail_b64.to_string(),
                                                };
                                                playlists_clone
                                                    .lock()
                                                    .unwrap_nc(&nc_clone, "playlist")
                                                    .map(|mut playlists| {
                                                        let playlist = playlists
                                                            .entry(device_id.to_string())
                                                            .or_insert_with(Vec::new);
                                                        playlist.push(song_internal);
                                                        playlist_updated_event(
                                                            nc_clone.clone(),
                                                            device_id.to_string(),
                                                            playlist,
                                                        );
                                                        if playlist.len() == 1 {
                                                            new_song_at_position_0(
                                                                nc_clone,
                                                                device_id.to_string(),
                                                                playlist,
                                                            );
                                                        }
                                                    });
                                            }
                                            _ => {}
                                        }
                                    }
                                    None => {}
                                }
                            }
                            PlaylistCommandContent::InsertSong => {
                                let command = command.command_as_insert_song().unwrap();
                                let song = command
                                    .song()
                                    .ok_or("Song is none")
                                    .unwrap_nc(&nc_clone, "playlist");
                                let position = command.index();
                                match song {
                                    Some(song) => {
                                        let title = song
                                            .title()
                                            .ok_or("Title is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let url = song
                                            .url()
                                            .ok_or("Url is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let thumbnail_b64 = song
                                            .thumbnail_b64()
                                            .ok_or("Thumbnail is none")
                                            .unwrap_nc(&nc_clone, "playlist");

                                        match (title, url, thumbnail_b64) {
                                            (Some(title), Some(url), Some(thumbnail_b64)) => {
                                                let song_internal = SongInternal {
                                                    url: url.to_string(),
                                                    title: title.to_string(),
                                                    thumbnail_b64: thumbnail_b64.to_string(),
                                                };
                                                playlists_clone
                                                    .lock()
                                                    .unwrap_nc(&nc_clone, "playlist")
                                                    .map(|mut playlists| {
                                                        let playlist = playlists
                                                            .entry(device_id.to_string())
                                                            .or_insert_with(Vec::new);
                                                        playlist.insert(
                                                            position as usize,
                                                            song_internal,
                                                        );
                                                        playlist_updated_event(
                                                            nc_clone.clone(),
                                                            device_id.to_string(),
                                                            playlist,
                                                        );
                                                        if position == 0 {
                                                            new_song_at_position_0(
                                                                nc_clone,
                                                                device_id.to_string(),
                                                                playlist,
                                                            );
                                                        }
                                                    });
                                            }
                                            _ => {}
                                        }
                                    }
                                    None => {}
                                }
                            }
                            PlaylistCommandContent::RemoveSong => {
                                let command = command.command_as_remove_song().unwrap();
                                let position = command.index();
                                playlists_clone.lock().unwrap_nc(&nc_clone, "playlist").map(
                                    |mut playlists| {
                                        let playlist = playlists
                                            .entry(device_id.to_string())
                                            .or_insert_with(Vec::new);
                                        playlist.remove(position as usize);
                                        playlist_updated_event(
                                            nc_clone.clone(),
                                            device_id.to_string(),
                                            playlist,
                                        );
                                        if position == 0 && !playlist.is_empty() {
                                            new_song_at_position_0(
                                                nc_clone.clone(),
                                                device_id.to_string(),
                                                playlist,
                                            );
                                        }
                                        if position == 0 && playlist.is_empty() {
                                            stop_command(nc_clone, device_id.to_string());
                                        }
                                    },
                                );
                            }
                            PlaylistCommandContent::ReplaceSong => {
                                let command = command.command_as_replace_song().unwrap();
                                let song = command
                                    .song()
                                    .ok_or("Song is none")
                                    .unwrap_nc(&nc_clone, "playlist");
                                let position = command.index();
                                match song {
                                    Some(song) => {
                                        let title = song
                                            .title()
                                            .ok_or("Title is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let url = song
                                            .url()
                                            .ok_or("Url is none")
                                            .unwrap_nc(&nc_clone, "playlist");
                                        let thumbnail_b64 = song
                                            .thumbnail_b64()
                                            .ok_or("Thumbnail is none")
                                            .unwrap_nc(&nc_clone, "playlist");

                                        match (title, url, thumbnail_b64) {
                                            (Some(title), Some(url), Some(thumbnail_b64)) => {
                                                let song_internal = SongInternal {
                                                    url: url.to_string(),
                                                    title: title.to_string(),
                                                    thumbnail_b64: thumbnail_b64.to_string(),
                                                };
                                                playlists_clone
                                                    .lock()
                                                    .unwrap_nc(&nc_clone, "playlist")
                                                    .map(|mut playlists| {
                                                        let playlist = playlists
                                                            .entry(device_id.to_string())
                                                            .or_insert_with(Vec::new);
                                                        if (position as usize) < playlist.len() {
                                                            playlist[position as usize] =
                                                                song_internal;
                                                        } else {
                                                            playlist.push(song_internal);
                                                        }
                                                        if position == 0 {
                                                            new_song_at_position_0(
                                                                nc_clone,
                                                                device_id.to_string(),
                                                                playlist,
                                                            );
                                                        }
                                                    });
                                            }
                                            _ => {}
                                        }
                                    }
                                    None => {}
                                }
                            }
                            PlaylistCommandContent(MAX..=u8::MAX) => {
                                send_error_message(&nc_clone, "Invalid command type", "playlist");
                            }
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
    });

    let playlists_clone2 = playlists.clone();

    let speaker_thread = thread::spawn(move || {
        for msg in sub_speaker.messages() {
            //listen for speaker events
            let nc_clone = nc.clone();
            let message = root_as_message(msg.data.as_slice()).unwrap_nc(&nc_clone, "speaker");
            match message {
                Some(message) => match message.content_type() {
                    MessageContent::SpeakerEvent => {
                        let event = message.content_as_speaker_event().unwrap();
                        let event_type = event.event_type();
                        let device_id = event
                            .device_id()
                            .ok_or("Device id is none")
                            .unwrap_nc(&nc_clone, "speaker");

                        match device_id {
                            Some(device_id) => match event_type {
                                SpeakerEventContent::FileEnded => {
                                    playlists_clone2.lock().unwrap_nc(&nc_clone, "speaker").map(
                                        |mut playlists| {
                                            let playlist = playlists
                                                .entry(device_id.to_string())
                                                .or_insert_with(Vec::new);
                                            if !playlist.is_empty() {
                                                playlist.remove(0);
                                                playlist_updated_event(
                                                    nc_clone.clone(),
                                                    device_id.to_string(),
                                                    playlist,
                                                );
                                                if !playlist.is_empty() {
                                                    new_song_at_position_0(
                                                        nc_clone,
                                                        device_id.to_string(),
                                                        playlist,
                                                    );
                                                } else {
                                                    stop_command(nc_clone, device_id.to_string());
                                                }
                                            }
                                        },
                                    );
                                }
                                _ => {}
                            },
                            None => {
                                send_error_message(&nc_clone, "Device id is none", "playlist");
                            }
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
    });

    // Wait for both threads to finish (if they are meant to finish)
    playlist_thread
        .join()
        .expect("Failed to join playlist thread");
    speaker_thread
        .join()
        .expect("Failed to join speaker thread");

    Ok(())
}
