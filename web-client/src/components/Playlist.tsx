import { Box, Button } from "@mantine/core";
import { Song } from "../types/song";
import { MusicEntry } from "./MusicEntry";
import FeatherIcon from "feather-icons-react";

export const Playlist = ({
  speaker,
  playlist,
  remove,
}: {
  speaker: string;
  playlist: Song[];
  remove: (index: number) => void;
}) => {
  const resume = async () => {
    // MauriceApi.postApiResume({
    //   speaker,
    // });
  };

  const pause = async () => {
    // MauriceApi.postApiPause({
    //   speaker,
    // });
  };

  const stop = async () => {
    // MauriceApi.postApiStop({
    //   speaker,
    // });
  };

  const next = async () => {
    // MauriceApi.postApiStop({
    //   speaker,
    // });
  };

  return playlist.map((song, i) => (
    <MusicEntry
      speaker={speaker}
      key={song.url}
      song={song}
      showTime={i === 0}
      controls={(color) => (
        <>
          {i === 0 && (
            <>
              {playlist.length == 1 && (
                <Box>
                  <Button
                    style={{
                      height: 60,
                      width: 60,
                      color,
                    }}
                    onClick={stop}
                  >
                    <FeatherIcon icon="stop-circle" />
                  </Button>
                </Box>
              )}
              <Box>
                <Button
                  style={{
                    height: 60,
                    width: 60,
                    color,
                  }}
                  onClick={pause}
                >
                  <FeatherIcon icon="pause" />
                </Button>
              </Box>
              <Box>
                <Button
                  style={{
                    height: 60,
                    width: 60,
                    color,
                  }}
                  onClick={resume}
                >
                  <FeatherIcon icon="play" />
                </Button>
              </Box>
              {playlist.length > 1 && (
                <Box>
                  <Button
                    style={{
                      height: 60,
                      width: 60,
                      color,
                    }}
                    onClick={next}
                  >
                    <FeatherIcon icon="skip-forward" />
                  </Button>
                </Box>
              )}
            </>
          )}
          {i !== 0 && (
            <Box>
              <Button
                style={{
                  height: 60,
                  width: 60,
                  color,
                }}
                onClick={() => remove(i)}
              >
                <FeatherIcon icon="trash" />
              </Button>
            </Box>
          )}
        </>
      )}
    />
  ));
};
