import { useEffect, useState } from "react";
import {
  constructPlayYoutubeMessage,
  constructQueryDeviceListMessage,
  sendMessage,
  subscribe,
} from "../nats/speakersNats";
import { Message } from "../schemas/message";
import { ByteBuffer } from "flatbuffers";
import { Echo } from "../schemas/echo";
import { useNats } from "../nats/NatsProvider";

export const NatsTestPage = () => {
  const [messages, setMessages] = useState<string[]>([]);
  const [deviceId, setDeviceId] = useState<string>("");
  const [url, setUrl] = useState<string>("");
  const nc = useNats();

  useEffect(() => {
    const unsubscribe = subscribe(nc, "echo", (msg) => {
      const buf = new ByteBuffer(msg);
      const message = Message.getRootAsMessage(buf);
      const content: Echo = message.content(new Echo());
      setMessages((messages) => [...messages, content.message() || ""]);
    });
    return unsubscribe;
  }, []);

  return (
    <div>
      <h1>NatsTest</h1>
      <button
        onClick={() => {
          const message = constructQueryDeviceListMessage();
          sendMessage(nc, "speaker.query", message);
        }}
      >
        Query Device List
      </button>
      <input
        type="text"
        value={deviceId}
        onChange={(e) => setDeviceId(e.target.value)}
        placeholder="Device ID"
      />
      <input
        type="text"
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="URL"
      />
      <button
        onClick={() => {
          const message = constructPlayYoutubeMessage(url, deviceId);
          sendMessage(nc, "speaker.command", message);
        }}
      >
        Play Youtube
      </button>
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "8px",
          maxHeight: "400px",
          overflowY: "auto",
        }}
      >
        {[...messages].reverse().map((message, index) => (
          <div key={index}>{message}</div>
        ))}
      </div>
    </div>
  );
};
