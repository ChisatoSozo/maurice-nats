import { Builder, ByteBuffer } from "flatbuffers";
import { Message } from "../schemas/message";
import { MessageContent } from "../schemas/message-content";
import { SpeakerCommand } from "../schemas/speaker-command";
import { SpeakerCommandContent } from "../schemas/speaker-command-content";
import { SpeakerListQuery } from "../schemas/speaker-list-query";
import { NatsConnection } from "@nats-io/nats-core";
import { Stop } from "../schemas/stop";
import { TogglePause } from "../schemas/toggle-pause";
import { QueryMusicVolume } from "../schemas/query-music-volume";
import { SpeakerQuery } from "../schemas/speaker-query";
import { SpeakerQueryContent } from "../schemas/speaker-query-content";
import { PlaylistCommand } from "../schemas/playlist-command";
import { PlaylistCommandContent } from "../schemas/playlist-command-content";
import { ReplaceSong } from "../schemas/replace-song";
import { Song } from "../schemas/song";
import { AddSong } from "../schemas/add-song";
import { PlaylistQuery } from "../schemas/playlist-query";
import { PlaylistQueryContent } from "../schemas/playlist-query-content";
import { QueryPlaylistState } from "../schemas/query-playlist-state";
import { Seek } from "../schemas/seek";
import { RemoveSong } from "../schemas/remove-song";
import { SetMusicVolume } from "../schemas/set-music-volume";

export const constructPlaySongMessage = (
  url: string,
  thumbnail: string,
  title: string,
  deviceId: string
) => {
  const builder = new Builder(1024);

  const song = Song.createSong(
    builder,
    builder.createString(url),
    builder.createString(thumbnail),
    builder.createString(title)
  );

  ReplaceSong.startReplaceSong(builder);
  ReplaceSong.addSong(builder, song);
  ReplaceSong.addIndex(builder, 0);
  const replaceSongOffset = ReplaceSong.endReplaceSong(builder);

  const contentOffset = PlaylistCommand.createPlaylistCommand(
    builder,
    builder.createString(deviceId),
    PlaylistCommandContent.ReplaceSong,
    replaceSongOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.PlaylistCommand,
    contentOffset
  );

  builder.finish(messageOffset);
  return builder.asUint8Array();
};

export const constructAddSongMessage = (
  url: string,
  thumbnail: string,
  title: string,
  deviceId: string
) => {
  const builder = new Builder(1024);

  const song = Song.createSong(
    builder,
    builder.createString(url),
    builder.createString(thumbnail),
    builder.createString(title)
  );

  const addSongOffset = AddSong.createAddSong(builder, song);

  const contentOffset = PlaylistCommand.createPlaylistCommand(
    builder,
    builder.createString(deviceId),
    PlaylistCommandContent.AddSong,
    addSongOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.PlaylistCommand,
    contentOffset
  );

  builder.finish(messageOffset);
  return builder.asUint8Array();
};

export const constructRemoveSongMessage = (deviceId: string, index: number) => {
  const builder = new Builder(1024);

  const removeSongOffset = RemoveSong.createRemoveSong(builder, index);

  const contentOffset = PlaylistCommand.createPlaylistCommand(
    builder,
    builder.createString(deviceId),
    PlaylistCommandContent.RemoveSong,
    removeSongOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.PlaylistCommand,
    contentOffset
  );

  builder.finish(messageOffset);
  return builder.asUint8Array();
};

export const constructStopMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const playOffset = Stop.createStop(builder);

  const contentOffset = SpeakerCommand.createSpeakerCommand(
    builder,
    builder.createString(deviceId),
    SpeakerCommandContent.Stop,
    playOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerCommand,
    contentOffset
  );

  builder.finish(messageOffset);
  return builder.asUint8Array();
};

export const constructTogglePauseMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const togglePauseOffset = TogglePause.createTogglePause(builder);

  const contentOffset = SpeakerCommand.createSpeakerCommand(
    builder,
    builder.createString(deviceId),
    SpeakerCommandContent.TogglePause,
    togglePauseOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerCommand,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructQueryVolumeMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const queryVolumeOffset = QueryMusicVolume.createQueryMusicVolume(builder);

  const contentOffset = SpeakerQuery.createSpeakerQuery(
    builder,
    builder.createString(deviceId),
    SpeakerQueryContent.QueryMusicVolume,
    queryVolumeOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerQuery,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructQuerySeekMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const queryVolumeOffset = QueryMusicVolume.createQueryMusicVolume(builder);

  const contentOffset = SpeakerQuery.createSpeakerQuery(
    builder,
    builder.createString(deviceId),
    SpeakerQueryContent.QuerySeek,
    queryVolumeOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerQuery,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructQueryDurationMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const queryVolumeOffset = QueryMusicVolume.createQueryMusicVolume(builder);

  const contentOffset = SpeakerQuery.createSpeakerQuery(
    builder,
    builder.createString(deviceId),
    SpeakerQueryContent.QueryDuration,
    queryVolumeOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerQuery,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructQueryPlaylistStateMessage = (deviceId: string) => {
  const builder = new Builder(1024);

  const queryPlaylistState =
    QueryPlaylistState.createQueryPlaylistState(builder);

  const contentOffset = PlaylistQuery.createPlaylistQuery(
    builder,
    builder.createString(deviceId),
    PlaylistQueryContent.QueryPlaylistState,
    queryPlaylistState
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.PlaylistQuery,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructQueryDeviceListMessage = () => {
  const builder = new Builder(1024);
  const contentOffset = SpeakerListQuery.createSpeakerListQuery(builder);

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerListQuery,
    contentOffset
  );

  builder.finish(messageOffset);
  return builder.asUint8Array();
};

export const constructSeekMessage = (deviceId: string, seek: number) => {
  const builder = new Builder(1024);

  const seekOffset = Seek.createSeek(builder, seek);

  const contentOffset = SpeakerCommand.createSpeakerCommand(
    builder,
    builder.createString(deviceId),
    SpeakerCommandContent.Seek,
    seekOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerCommand,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const constructVolumeMessage = (deviceId: string, volume: number) => {
  const builder = new Builder(1024);

  const volumeOffset = SetMusicVolume.createSetMusicVolume(builder, volume);

  const contentOffset = SpeakerCommand.createSpeakerCommand(
    builder,
    builder.createString(deviceId),
    SpeakerCommandContent.SetMusicVolume,
    volumeOffset
  );

  const messageOffset = Message.createMessage(
    builder,
    BigInt(Date.now()),
    MessageContent.SpeakerCommand,
    contentOffset
  );

  builder.finish(messageOffset);

  return builder.asUint8Array();
};

export const sendMessage = (
  nc: NatsConnection,
  topic: string,
  message: Uint8Array
) => {
  nc.publish(topic, message);
};

export const subscribe = (
  nc: NatsConnection,
  topic: string,
  callback: (msg: Message) => void
) => {
  const subscription = nc.subscribe(topic, {
    callback: (err, msg) => {
      if (err) {
        console.error(err);
      } else {
        const buffer = new ByteBuffer(msg.data);
        const message = Message.getRootAsMessage(buffer);
        callback(message);
      }
    },
  });
  return () => {
    subscription.unsubscribe();
  };
};
