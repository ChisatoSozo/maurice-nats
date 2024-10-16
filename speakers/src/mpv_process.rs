use crate::fbs::construct_duration_changed_event_message;
use crate::fbs::construct_file_ended_event_message;
use crate::fbs::construct_seek_changed_event_message;
use crate::fbs::NcSendable;
use serde_json::Value;
use std::io::BufRead;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::fbs::construct_volume_changed_event_message;

pub struct MpvProcess {
    process: std::process::Child,
    socket: UnixStream,
    kill_signal: Arc<AtomicBool>,
}

const PROPERTY_MAP: &[(&str, u64)] = &[
    ("volume", 1),
    ("mute", 2),
    ("pause", 3),
    ("time-pos", 4),
    ("duration", 5),
    ("filename", 6),
];

impl MpvProcess {
    pub fn new<'a>(
        primary_arg: &str,
        device_id: &str,
        nc: Arc<nats::Connection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let time_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let socket_path = format!("/tmp/mpv-socket-{}", time_since_epoch.as_millis());
        println!("starting mpv with socket path: {}", socket_path);
        let mpv_process = std::process::Command::new("mpv")
            .arg(primary_arg)
            .arg("--no-video")
            .arg(format!("--audio-device=alsa/{device_id}"))
            .arg(format!("--input-ipc-server={}", socket_path))
            .spawn()
            .map_err(|e| e.to_string())?;

        println!("Waiting for mpv socket to be created...");
        let timeout = std::time::Duration::from_secs(5);
        let start = Instant::now();
        loop {
            if start.elapsed() >= timeout {
                return Err("Timeout waiting for mpv socket".into());
            }

            if std::fs::metadata(&socket_path).is_ok() {
                break;
            }

            println!("Waiting for mpv socket to be created...");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!("Mpv socket created.");
        let socket = UnixStream::connect(socket_path)?;

        println!("socket connected");
        let kill_signal = Arc::new(AtomicBool::new(false));

        println!("Creating recv_thread...");
        Self::make_recv_thread(socket.try_clone()?, nc, Arc::clone(&kill_signal), device_id);
        println!("recv_thread created.");

        Ok(Self {
            process: mpv_process,
            socket,
            kill_signal,
        })
    }

    pub fn kill(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //is the process already dead?
        if self.process.try_wait()?.is_some() {
            return Ok(());
        }

        // Set the kill signal to true to notify the recv_thread to stop
        println!("Setting kill signal to true...");
        self.kill_signal.store(true, Ordering::SeqCst);

        println!("Shutting down socket and killing mpv process...");
        // Shutdown the socket (this will interrupt any blocking read)
        self.socket.shutdown(std::net::Shutdown::Both)?;

        // Kill the mpv process
        println!("Killing mpv process...");
        self.process.kill()?;
        println!("Waiting for mpv process to terminate...");
        self.process.wait()?; // Ensure process is fully terminated
        println!("Mpv process terminated.");

        Ok(())
    }

    pub fn query_property(&mut self, property: &str) -> Result<(), Box<dyn std::error::Error>> {
        let property_number = PROPERTY_MAP
            .iter()
            .find(|(name, _)| name == &property)
            .map(|(_, number)| *number)
            .ok_or("Property not found")?;
        let request_id = property_number;

        let query = format!(
            "{{\"command\":[\"get_property\",\"{}\"],\"request_id\":{}}}\n",
            property, request_id
        );
        self.socket.write_all(query.as_bytes())?;

        Ok(())
    }

    pub fn set_property(
        &mut self,
        property: &str,
        value: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let property_number = PROPERTY_MAP
            .iter()
            .find(|(name, _)| name == &property)
            .map(|(_, number)| *number)
            .ok_or("Property not found")?;
        let request_id = property_number;

        let query = format!(
            "{{\"command\":[\"set_property\",\"{}\",{}],\"request_id\":{}}}\n",
            property, value, request_id
        );
        self.socket.write_all(query.as_bytes())?;

        Ok(())
    }

    fn make_recv_thread(
        socket: UnixStream,
        nc: Arc<nats::Connection>,
        kill_signal: Arc<AtomicBool>,
        device_id: &str,
    ) -> JoinHandle<()> {
        let device_id = device_id.to_string();
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(socket);
            loop {
                // Check if the kill signal has been set
                if kill_signal.load(Ordering::SeqCst) {
                    println!("Kill signal received. Exiting recv_thread...");
                    break;
                }

                let mut buffer = Vec::new();
                match reader.read_until(b'\n', &mut buffer) {
                    Ok(0) => {
                        // EOF reached, break the loop
                        println!("EOF reached on the socket.");
                        break;
                    }
                    Ok(_) => {
                        // Attempt to parse the received data as JSON
                        match serde_json::from_slice::<Value>(&buffer) {
                            Ok(event_obj) => {
                                let event = &event_obj["event"];
                                let data = &event_obj["data"];
                                let request_id = &event_obj["request_id"];
                                if let Value::String(event) = event {
                                    match event.as_str() {
                                        "end-file" => {
                                            println!("End of file reached.");
                                            let message = Ok(Some(
                                                construct_file_ended_event_message(&device_id),
                                            ));
                                            message.send(&nc, "speaker.event");
                                        }
                                        _ => {
                                            println!("Received event: {}", event);
                                        }
                                    }
                                } else if let (Value::Number(data), Value::Number(request_id)) =
                                    (data, request_id)
                                {
                                    let property_name = PROPERTY_MAP
                                        .iter()
                                        .find(|(_, number)| number == &request_id.as_u64().unwrap())
                                        .map(|(name, _)| name)
                                        .unwrap_or(&"Unknown");

                                    match property_name {
                                        &"volume" => {
                                            let volume = data.as_f64().unwrap();
                                            println!("Received volume: {}", volume);
                                            let message =
                                                Ok(Some(construct_volume_changed_event_message(
                                                    volume as f32,
                                                    &device_id,
                                                )));
                                            message.send(&nc, "speaker.event");
                                        }
                                        &"time-pos" => {
                                            let time_pos = data.as_f64().unwrap();
                                            println!("Received time-pos: {}", time_pos);
                                            let message =
                                                Ok(Some(construct_seek_changed_event_message(
                                                    time_pos as f32,
                                                    &device_id,
                                                )));
                                            message.send(&nc, "speaker.event");
                                        }
                                        &"duration" => {
                                            let duration = data.as_f64().unwrap();
                                            println!("Received duration: {}", duration);
                                            let message =
                                                Ok(Some(construct_duration_changed_event_message(
                                                    duration as f32,
                                                    &device_id,
                                                )));
                                            message.send(&nc, "speaker.event");
                                        }
                                        _ => {
                                            println!(
                                                "Received property: {} with value: {}",
                                                property_name, data
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error parsing JSON: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from mpv socket: {}", e);
                        break;
                    }
                }
            }
        })
    }
}
