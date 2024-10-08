// import { wsconnect } from "@nats-io/nats-core";
// import { Message } from "./schemas/message";
// import { Builder } from "flatbuffers";
// import { Print } from "./schemas/print";
// import { MessageContent } from "./schemas/message-content";
import {
  createBrowserRouter,
  Outlet,
  RouteObject,
  RouterProvider,
} from "react-router-dom";
import { UserSelectPage } from "./pages/UserSelectPage";
import { ScaffoldPage } from "./pages/ScaffoldPage";
import { HomePage } from "./pages/HomePage";
import { AudioPage } from "./pages/AudioPage";
import { NatsTestPage } from "./pages/NatsTestPage";
import { LogoutPage } from "./pages/LogoutPage";
import { SpeakerPage } from "./pages/SpeakerPage";
import { NatsProvider } from "./nats/NatsProvider";

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

//   const nc = await wsconnect({
//     servers: ["ws://192.168.2.56:8080"],
//   });
//   nc.publish("print", buf);
// };

// hi();

const routes: RouteObject[] = [
  {
    path: "/",
    element: <Outlet />,
    children: [
      {
        path: "",
        element: <UserSelectPage />,
      },
      {
        path: ":user",
        element: <ScaffoldPage />,
        children: [
          {
            path: "",
            element: <HomePage />,
          },
          {
            path: "audio",
            element: <AudioPage />,
          },
          {
            path: "audio/:speaker",
            element: <SpeakerPage />,
          },
          {
            path: "nats_test",
            element: <NatsTestPage />,
          },
          {
            path: "logout",
            element: <LogoutPage />,
          },
        ],
      },
    ],
  },
];

const router = createBrowserRouter(routes);

export const App = () => {
  return (
    <NatsProvider>
      <RouterProvider router={router} />
    </NatsProvider>
  );
};
