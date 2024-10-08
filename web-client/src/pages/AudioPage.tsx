import { IconAlertCircle, IconDeviceSpeaker } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import {
  constructQueryDeviceListMessage,
  sendMessage,
  subscribe,
} from "../nats/speakersNats";
import { ByteBuffer } from "flatbuffers";
import { Message } from "../schemas/message";
import { MessageContent } from "../schemas/message-content";
import { SpeakerListEvent } from "../schemas/speaker-list-event";
import { Button, Grid, GridCol } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import { useNats } from "../nats/NatsProvider";

const knownDevices: Record<
  string,
  {
    name: string;
    icon: React.ReactNode;
  }
> = {
  "plughw:CARD=MC1000,DEV=0": {
    name: "Don't use or I will hurt you",
    icon: <IconAlertCircle />,
  },
  "plughw:CARD=Generic,DEV=0": {
    name: "Main Speaker",
    icon: <IconDeviceSpeaker />,
  },
};

export const AudioPage = () => {
  const [speakers, setSpeakers] = useState<string[]>([]);
  const navigate = useNavigate();
  const nc = useNats();

  useEffect(() => {
    sendMessage(nc, "speaker.query", constructQueryDeviceListMessage());
    const unsubscribe = subscribe(nc, "speaker.event", (msg) => {
      const buf = new ByteBuffer(msg);
      const message = Message.getRootAsMessage(buf);
      const type = message.contentType();

      if (type == MessageContent.SpeakerListEvent) {
        const content: SpeakerListEvent = message.content(
          new SpeakerListEvent()
        );
        const devLength = content.deviceIdsLength();
        const devices = [];
        for (let i = 0; i < devLength; i++) {
          const deviceId = content.deviceIds(i);
          devices.push(deviceId);
        }
        setSpeakers(devices);
      }
    });
    return unsubscribe;
  }, [nc]);

  return (
    <Grid>
      {speakers.map((speaker) => (
        <GridCol span={6} key={speaker}>
          <Button onClick={() => navigate(encodeURIComponent(speaker))}>
            {knownDevices[speaker]?.icon || <IconDeviceSpeaker />}
            {knownDevices[speaker]?.name || speaker}
          </Button>
        </GridCol>
      ))}
    </Grid>
  );
};
