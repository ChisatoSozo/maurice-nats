import { Builder } from "flatbuffers";
import { PlayYoutube } from "../schemas/play-youtube";
import { Message } from "../schemas/message";
import { MessageContent } from "../schemas/message-content";
import { SpeakerCommand } from "../schemas/speaker-command";
import { SpeakerCommandContent } from "../schemas/speaker-command-content";
import { PlayContent } from "../schemas/play-content";
import { Play } from "../schemas/play";
import { SpeakerListQuery } from "../schemas/speaker-list-query";
import { NatsConnection, wsconnect } from "@nats-io/nats-core";
import { Stop } from "../schemas/stop";

export const constructPlayYoutubeMessage = (url: string, deviceId: string) => {
  const builder = new Builder(1024);
  const playYoutubeOffset = PlayYoutube.createPlayYoutube(
    builder,
    builder.createString(url)
  );

  const playOffset = Play.createPlay(
    builder,
    PlayContent.PlayYoutube,
    playYoutubeOffset
  );

  const contentOffset = SpeakerCommand.createSpeakerCommand(
    builder,
    builder.createString(deviceId),
    SpeakerCommandContent.Play,
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
  callback: (msg: Uint8Array) => void
) => {
  const subscription = nc.subscribe(topic, {
    callback: (err, msg) => {
      if (err) {
        console.error(err);
      } else {
        callback(msg.data);
      }
    },
  });
  return () => {
    subscription.unsubscribe();
  };
};

// const hi = async () => {
//   const builder = new Builder(1024);
//   const printOffset = Print.createPrint(
//     builder,
//     builder.createString("Hello from browser")
//   );
//   Message.startMessage(builder);
//   Message.addTimestamp(builder, BigInt(Date.now()));
//   Message.addContentType(builder, MessageContent.Print);
//   Message.addContent(builder, printOffset);
//   const messageOffset = Message.endMessage(builder);
//   builder.finish(messageOffset);
//   const buf = builder.asUint8Array();

//   nc.publish("print", buf);
// };
