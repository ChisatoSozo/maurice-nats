import {
  AppShell,
  Box,
  Burger,
  Button,
  Group,
  Image,
  Text,
  useMantineTheme,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import FeatherIcon from "feather-icons-react";
import { Outlet, useNavigate, useParams, useLocation } from "react-router-dom";

const NavButton = ({
  icon,
  name,
  path,
}: {
  icon: React.ReactNode;
  name: string;
  path: string;
}) => {
  const navigate = useNavigate();
  const params = useParams();
  const location = useLocation();
  const theme = useMantineTheme();

  let isActive = location.pathname.startsWith("/" + params.user + path);

  if (path === "/") {
    isActive = location.pathname === "/" + params.user + path;
  }

  return (
    <Button
      fullWidth
      variant="default"
      style={{
        display: "flex",
        justifyContent: "flex-start",
        borderRadius: 0,
        border: 0,
        outline: 0,
        color: isActive ? theme.colors.blue[8] : "inherit",
      }}
      onClick={() => navigate("/" + params.user + path)}
    >
      <Group>
        {icon}
        <Text size="lg">{name}</Text>
      </Group>
    </Button>
  );
};

export const ScaffoldPage = () => {
  const [opened, { toggle }] = useDisclosure();
  const params = useParams();

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{
        width: 300,
        breakpoint: "sm",
        collapsed: { mobile: !opened },
      }}
      padding="md"
    >
      <AppShell.Header
        style={{
          display: "flex",
          alignItems: "center",
          gap: 16,
          padding: "0 16px",
        }}
      >
        <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />

        <Image
          src={"/web-app-manifest-192x192.png"}
          alt="Logo"
          style={{ borderRadius: "50%", width: 40, height: 40 }}
        />
      </AppShell.Header>

      <AppShell.Navbar
        p="md"
        style={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
        }}
      >
        <Box>
          <NavButton icon={<FeatherIcon icon="home" />} name="Home" path="/" />
          <NavButton
            icon={<FeatherIcon icon="speaker" />}
            name="Audio"
            path="/audio"
          />
          <NavButton
            icon={<FeatherIcon icon="code" />}
            name="Nats Test"
            path="/nats_test"
          />
        </Box>
        <Box>
          <NavButton
            icon={<FeatherIcon icon="log-out" />}
            name="Logout"
            path="/logout"
          />
          <Text
            size="sm"
            style={{
              width: "100%",
              textAlign: "right",
            }}
          >
            Logged in as <strong>{params.user}</strong>
          </Text>
        </Box>
      </AppShell.Navbar>

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
};
