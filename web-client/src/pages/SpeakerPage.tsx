import { Button, Input } from "@mantine/core";
import { useState } from "react";
import { useParams } from "react-router-dom";
import {
  constructPlayYoutubeMessage,
  constructStopMessage,
  sendMessage,
} from "../nats/speakersNats";
import { useNats } from "../nats/NatsProvider";

export const SpeakerPage = () => {
  const params = useParams();
  const [url, setUrl] = useState("");
  const speaker = decodeURIComponent(params.speaker || "");
  const nc = useNats();

  return (
    <>
      <Input
        value={url}
        onChange={(e) => setUrl(e.currentTarget.value)}
        placeholder="Youtube URL"
      />
      <Button
        onClick={() => {
          sendMessage(
            nc,
            "speaker.command",
            constructPlayYoutubeMessage(url, speaker)
          );
        }}
      >
        Play
      </Button>
      <Button
        onClick={() => {
          sendMessage(nc, "speaker.command", constructStopMessage(speaker));
        }}
      >
        Stop
      </Button>
    </>
  );
};
