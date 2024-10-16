import { Box, Button, Input } from "@mantine/core";
import { useCallback, useState } from "react";
import { MusicEntry } from "./MusicEntry";
import { Song } from "../types/song";
import FeatherIcon from "feather-icons-react";

export const GetYoutubeSong = ({
  onAddSong,
  onPlaySong,
  speaker,
}: {
  onAddSong?: (song: Song) => void;
  onPlaySong?: (song: Song) => void;
  speaker: string;
}) => {
  const [searchValue, setSearchValue] = useState("");
  const [songs, setSongs] = useState<Song[]>([]);
  const [loading, setLoading] = useState(false);

  //tes
  const search = useCallback(async () => {
    if (!searchValue) {
      return;
    }
    setLoading(true);
    const videosResponse = await fetch(
      "http://192.168.2.56:3000/api/get_youtube_videos",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          search: searchValue,
        }),
      }
    ).then((res) => res.json());

    setLoading(false);
    setSongs(videosResponse.videos);
  }, [searchValue]);

  return (
    <Box
      display="flex"
      style={{
        flexDirection: "column",
        gap: 1,
      }}
    >
      <Box
        display="flex"
        style={{
          gap: 10,
        }}
        mt={10}
        mb={10}
      >
        <Input
          placeholder="Enter a youtube song name"
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              search();
            }
          }}
          onChange={(e) => {
            setSearchValue(e.target.value);
          }}
          value={searchValue}
          style={{
            width: "100%",
          }}
        />
        <Button
          disabled={!searchValue || loading}
          onClick={search}
          style={{
            width: 130,
          }}
        >
          {loading ? "Loading..." : "Search"}
        </Button>
      </Box>
      {songs.map((song) => (
        <MusicEntry
          speaker={speaker}
          song={song}
          controls={(color) => (
            <>
              <Box
                style={{
                  marginRight: 10,
                }}
              >
                <Button
                  style={{
                    height: 60,
                    width: 60,
                    color: color,
                  }}
                  variant="transparent"
                  onClick={() => onAddSong?.(song)}
                >
                  <FeatherIcon icon="plus" />
                </Button>
              </Box>
              <Box>
                <Button
                  style={{
                    height: 60,
                    width: 60,
                    color,
                  }}
                  variant="transparent"
                  onClick={() => onPlaySong?.(song)}
                >
                  <FeatherIcon icon="play" />
                </Button>
              </Box>
            </>
          )}
        />
      ))}
    </Box>
  );
};
