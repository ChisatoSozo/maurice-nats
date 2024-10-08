import { NatsConnection, wsconnect } from "@nats-io/nats-core";
import { createContext, useContext, useEffect, useState } from "react";

interface NatsContextProps {
  nc: NatsConnection;
}

export const NatsContext = createContext<NatsContextProps | null>(null);

export const NatsProvider = ({ children }: { children: React.ReactNode }) => {
  const [nc, setNc] = useState<NatsConnection | undefined>(undefined);

  useEffect(() => {
    const getNatsConnection = async () => {
      const nc = await wsconnect({
        servers: ["ws://192.168.2.56:8080"],
      });
      setNc(nc);
    };
    getNatsConnection();
  }, []);

  if (nc === undefined) {
    return "Loading...";
  }

  return <NatsContext.Provider value={{ nc }}>{children}</NatsContext.Provider>;
};

export const useNats = () => {
  const context = useContext(NatsContext);
  if (context === null) {
    throw new Error("useNats must be used within a NatsProvider");
  }
  return context.nc;
};
