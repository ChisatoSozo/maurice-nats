import { useParams } from "react-router-dom";

import { GetYoutubeSong } from "../components/GetYoutubeVideo";
import { useNats } from "../nats/NatsProvider";
import { useCallback, useEffect, useState } from "react";
import {
  constructAddSongMessage,
  constructPlaySongMessage,
  constructQueryDurationMessage,
  constructQueryPlaylistStateMessage,
  constructQuerySeekMessage,
  constructQueryVolumeMessage,
  constructRemoveSongMessage,
  constructSeekMessage,
  constructStopMessage,
  constructTogglePauseMessage,
  constructVolumeMessage,
  sendMessage,
  subscribe,
} from "../nats/speakersNats";
import { Box, Button, Slider, Text } from "@mantine/core";
import { Song } from "../types/song";
import { Message } from "../schemas/message";
import { MessageContent } from "../schemas/message-content";
import { PlaylistEvent } from "../schemas/playlist-event";
import { PlaylistEventContent } from "../schemas/playlist-event-content";
import { PlaylistStateChanged } from "../schemas/playlist-state-changed";
import { SpeakerEvent } from "../schemas/speaker-event";
import { SpeakerEventContent } from "../schemas/speaker-event-content";
import { MusicVolumeChanged } from "../schemas/music-volume-changed";
import { MusicEntry } from "../components/MusicEntry";
import { SeekChanged } from "../schemas/seek-changed";
import { DurationChanged } from "../schemas/duration-changed";
import FeatherIcon from "feather-icons-react";

export const SpeakerPage = () => {
  const params = useParams();
  const speaker = decodeURIComponent(params.speaker || "");
  const nc = useNats();

  const [songs, setSongs] = useState<Song[]>([]);
  const [internalVolumeState, setInternalVolumeState] = useState<number | null>(
    null
  );
  const [volume, setVolume] = useState(0);
  const [seek, setSeek] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    if (songs.length === 0) {
      return;
    }
    sendMessage(nc, "speaker.query", constructQueryDurationMessage(speaker));

    const interval = setInterval(() => {
      sendMessage(nc, "speaker.query", constructQuerySeekMessage(speaker));
    }, 500);

    return () => clearInterval(interval);
  }, [nc, speaker, songs]);

  useEffect(() => {
    sendMessage(nc, "speaker.query", constructQueryVolumeMessage(speaker));
  }, [nc, speaker]);

  useEffect(() => {
    sendMessage(
      nc,
      "playlist.query",
      constructQueryPlaylistStateMessage(speaker)
    );
  }, [nc, speaker]);

  useEffect(() => {
    const unsubPlaylist = subscribe(nc, "playlist.event", (msg) => {
      const type = msg.contentType();
      if (type == MessageContent.PlaylistEvent) {
        const event: PlaylistEvent = msg.content(new PlaylistEvent());
        const deviceId = event.deviceId();
        const type = event.eventType();
        if (deviceId !== speaker) {
          return;
        }
        if (type == PlaylistEventContent.PlaylistStateChanged) {
          const content: PlaylistStateChanged = event.event(
            new PlaylistStateChanged()
          );
          const songsLength = content.songsLength();

          const newSongs = [];
          for (let i = 0; i < songsLength; i++) {
            const song = content.songs(i);
            if (!song) {
              continue;
            }
            newSongs.push({
              url: song.url(),
              title: song.title(),
              thumbnail_b64: song.thumbnailB64(),
            } as Song);
          }
          setSongs(newSongs);
        }
      }
    });
    return unsubPlaylist;
  }, [nc, speaker]);

  useEffect(() => {
    //speaker volume event
    const unsubVolume = subscribe(nc, "speaker.event", (msg) => {
      const type = msg.contentType();
      if (type == MessageContent.SpeakerEvent) {
        const event: SpeakerEvent = msg.content(new SpeakerEvent());
        const deviceId = event.deviceId();
        if (deviceId !== speaker) {
          return;
        }
        const type = event.eventType();
        if (type == SpeakerEventContent.MusicVolumeChanged) {
          const volume: MusicVolumeChanged = event.event(
            new MusicVolumeChanged()
          );
          setVolume(volume.volume());
        }
        if (type == SpeakerEventContent.SeekChanged) {
          const seek: SeekChanged = event.event(new SeekChanged());
          setSeek(seek.seek());
        }
        if (type == SpeakerEventContent.DurationChanged) {
          const duration: DurationChanged = event.event(new DurationChanged());
          setDuration(duration.duration());
        }
      }
    });
    return unsubVolume;
  }, [nc, speaker]);

  const play = useCallback(
    (song: Song) => {
      sendMessage(
        nc,
        "playlist.command",
        constructPlaySongMessage(
          song.url,
          song.thumbnail_b64,
          song.title,
          speaker
        )
      );
    },
    [nc, speaker]
  );

  const add = useCallback(
    (song: Song) => {
      sendMessage(
        nc,
        "playlist.command",
        constructAddSongMessage(
          song.url,
          song.thumbnail_b64,
          song.title,
          speaker
        )
      );
    },
    [nc, speaker]
  );

  const remove = useCallback(
    (index: number) => {
      sendMessage(
        nc,
        "playlist.command",
        constructRemoveSongMessage(speaker, index)
      );
    },
    [speaker]
  );

  const sendSeek = useCallback(
    (time: number) => {
      sendMessage(nc, "speaker.command", constructSeekMessage(speaker, time));
    },
    [nc, speaker]
  );

  const pause = useCallback(() => {
    sendMessage(nc, "speaker.command", constructTogglePauseMessage(speaker));
  }, [nc, speaker]);

  useEffect(() => {
    if (internalVolumeState === null) {
      return;
    }
    sendMessage(
      nc,
      "speaker.command",
      constructVolumeMessage(speaker, internalVolumeState)
    );
  }, [nc, speaker, internalVolumeState]);

  return (
    <>
      {volume != null && (
        <Box
          display={"flex"}
          w="100%"
          pt="16"
          pb="16"
          style={{
            alignItems: "center",
            gap: 16,
          }}
        >
          <Text pb="4">Volume:</Text>
          <Slider w="100%" value={volume} onChange={setInternalVolumeState} />
        </Box>
      )}
      {songs.map((song, i) =>
        i === 0 ? (
          <MusicEntry
            showTime
            key={song.url}
            song={song}
            speaker={speaker}
            time={seek}
            maxTime={duration}
            updateTime={sendSeek}
            controls={(color) => (
              <>
                <Box>
                  <Button
                    variant="transparent"
                    style={{
                      height: 60,
                      width: 60,
                      color,
                    }}
                    onClick={() => remove(i)}
                  >
                    <FeatherIcon icon="skip-forward" />
                  </Button>
                  <Button
                    variant="transparent"
                    style={{
                      height: 60,
                      width: 60,
                      color,
                    }}
                    onClick={() => pause()}
                  >
                    <FeatherIcon icon="pause" />
                  </Button>
                </Box>
              </>
            )}
          />
        ) : (
          <MusicEntry
            key={song.url}
            song={song}
            speaker={speaker}
            controls={(color) => (
              <>
                <Box>
                  <Button
                    variant="transparent"
                    style={{
                      height: 60,
                      width: 60,
                      color,
                    }}
                    onClick={() => remove(i)}
                  >
                    <FeatherIcon icon="trash" />
                  </Button>
                </Box>
              </>
            )}
          />
        )
      )}
      <GetYoutubeSong speaker={speaker} onPlaySong={play} onAddSong={add} />
    </>
  );
};
