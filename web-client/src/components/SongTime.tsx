import { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Slider, Text } from "@mantine/core";

export const SongTime = ({
  time,
  maxTime,
  updateTime,
  color,
}: {
  time: number;
  maxTime: number;
  updateTime: (time: number) => void;
  color: string;
}) => {
  const [localTime, setLocalTime] = useState(time);

  useEffect(() => {
    setLocalTime(time);
  }, [time]);

  const setTime = useCallback(
    (time: number) => {
      setLocalTime(time);
      updateTime(time);
    },
    [updateTime]
  );

  //00:00, 03:00, 03:30, etc
  const formattedTime = useMemo(() => {
    const minutes = Math.floor(localTime / 60);
    const seconds = localTime - minutes * 60;

    return `${minutes.toFixed(0)}:${seconds < 10 ? "0" : ""}${seconds.toFixed(
      0
    )}`;
  }, [time]);

  const formattedMaxTime = useMemo(() => {
    const minutes = Math.floor(maxTime / 60);
    const seconds = maxTime - minutes * 60;

    return `${minutes.toFixed(0)}:${seconds < 10 ? "0" : ""}${seconds.toFixed(
      0
    )}`;
  }, [maxTime]);

  return (
    <Box
      display="flex"
      w="calc(100% - 10)"
      style={{
        alignItems: "center",
        gap: 2,
      }}
      pl={5}
      pr={5}
      pb={1}
    >
      <Slider
        value={time}
        max={maxTime}
        onChange={(value) => setTime(value as number)}
        style={{
          flex: 1,
        }}
      />
      <Text
        style={{
          width: 60,
          textAlign: "left",
          color: color,
        }}
      >
        {formattedTime}/{formattedMaxTime}
      </Text>
    </Box>
  );
};
