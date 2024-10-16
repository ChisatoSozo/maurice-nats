import { analyzeImageColors, ColorAnalysis } from "../utils/util";
import { useEffect, useState } from "react";
import { SongTime } from "./SongTime";
import { useMediaQuery } from "@mantine/hooks";
import { Song } from "../types/song";
import { Box, Text } from "@mantine/core";

export const MusicEntry = ({
  song,
  controls,
  showTime,
  time,
  maxTime,
  updateTime,
}: {
  song: Song;
  controls: (textColor: string) => React.ReactNode;
  speaker: string;
  showTime?: boolean;
  time?: number;
  maxTime?: number;
  updateTime?: (time: number) => void;
}) => {
  const [color, setColor] = useState<ColorAnalysis | null>(null);

  useEffect(() => {
    analyzeImageColors(song.thumbnail_b64).then(setColor);
  }, [song.thumbnail_b64]);

  const small = useMediaQuery("(max-width: 600px)");

  if (small) {
    return (
      <Box
        display={"flex"}
        style={{
          flexDirection: "column",
          height: showTime ? 220 : 160,
          borderRadius: 10,
          overflow: "hidden",
          backgroundColor: color?.dominant.rgb || "white",
        }}
      >
        <Box
          style={{
            display: "flex",
            flex: 1,
            flexDirection: "row",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <img
            src={song.thumbnail_b64}
            width={178}
            height={100}
            style={{
              objectFit: "cover",
            }}
          />
          <Box flex={1} h="100%">
            <Text p={1} c={color?.suggestedTextColor.rgb || "black"} h={80}>
              {song.title}
            </Text>
          </Box>
        </Box>
        <Box
          style={{
            display: "flex",
            flexDirection: "row",
            justifyContent: "space-between",
            alignItems: "center",
            width: 178,
          }}
        >
          {controls(color?.suggestedTextColor.rgb || "black")}
        </Box>
        {showTime && (
          <SongTime
            time={time || 0}
            maxTime={maxTime || 0}
            updateTime={updateTime || (() => {})}
            color={color?.suggestedTextColor.rgb || "black"}
          />
        )}
      </Box>
    );
  }

  return (
    <Box
      display={"flex"}
      style={{
        flexDirection: "column",
        height: showTime ? 160 : 100,
        borderRadius: 10,
        overflow: "hidden",
        backgroundColor: color?.dominant.rgb || "white",
      }}
    >
      <Box
        style={{
          display: "flex",
          flex: 1,
          flexDirection: "row",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <img
          src={song.thumbnail_b64}
          width={178}
          height={100}
          style={{
            objectFit: "cover",
          }}
        />
        <Box flex={1} h="100%">
          <Text p={1} c={color?.suggestedTextColor.rgb || "black"}>
            {song.title}
          </Text>
        </Box>

        {controls(color?.suggestedTextColor.rgb || "black")}
      </Box>
      {showTime && (
        <SongTime
          time={time || 0}
          maxTime={maxTime || 0}
          updateTime={updateTime || (() => {})}
          color={color?.suggestedTextColor.rgb || "black"}
        />
      )}
    </Box>
  );
};
