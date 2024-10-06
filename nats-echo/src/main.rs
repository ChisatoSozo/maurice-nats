extern crate flatbuffers;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_echo_generated.rs"]
mod msg_echo_generated;
use std::fs::OpenOptions;

pub use msg_echo_generated::*;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/msg_print_generated.rs"]
mod msg_print_generated;
pub use msg_print_generated::*;

#[allow(dead_code, unused_imports)]
#[path = "./schemas/root_generated.rs"]
mod root_generated;
pub use root_generated::*;

use std::io::Write;

const NATS_ECHO_SUBJECT: &str = "echo";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nc = nats::connect("nats://nats-server:4222")?;
    //listen for messages on all subjects
    let sub = nc.subscribe(">")?;

    for msg in sub.messages() {
        if msg.subject == NATS_ECHO_SUBJECT {
            continue;
        }

        let message = root_as_message(msg.data.as_slice())?;

        let timestamp = message.timestamp();

        let message_content_stringified: &str;
        let message_type = message.content_type();
        let message_type_stringified: &str;

        match message_type {
            MessageContent::NONE => {
                message_type_stringified = "NONE";
                message_content_stringified = "NONE";
            }
            MessageContent::Print => {
                message_type_stringified = "Print";
                let print_message = message.content_as_print();
                match print_message {
                    Some(print_message) => match print_message.message() {
                        Some(message) => {
                            message_content_stringified = message;
                        }
                        None => {
                            message_content_stringified = "{NO MESSAGE}";
                        }
                    },
                    None => {
                        message_content_stringified =
                            "{ERROR: TYPE WAS PRINT BUT content_as_print WAS NONE}";
                    }
                }
            }
            MessageContent::Echo => {
                message_type_stringified = "Echo";
                let echo_message = message.content_as_echo();
                match echo_message {
                    Some(echo_message) => match echo_message.message() {
                        Some(message) => {
                            message_content_stringified = message;
                        }
                        None => {
                            message_content_stringified = "{NO MESSAGE}";
                        }
                    },
                    None => {
                        message_content_stringified =
                            "{ERROR: TYPE WAS ECHO BUT content_as_echo WAS NONE}";
                    }
                }
            }
            _ => {
                message_type_stringified = "UNKNOWN";
                message_content_stringified = "UNKNOWN";
            }
        }

        let full_string = format!(
            "[{}] {}: {}",
            msg.subject, message_type_stringified, message_content_stringified
        );

        //construct an echo message with the full_string
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let message_string = fbb.create_string(full_string.as_str());
        let echo_message = Echo::create(
            &mut fbb,
            &EchoArgs {
                message: Some(message_string),
            },
        );
        let root = Message::create(
            &mut fbb,
            &MessageArgs {
                timestamp,
                content_type: MessageContent::Echo,
                content: Some(echo_message.as_union_value()),
            },
        );
        fbb.finish(root, None);

        //publish the echo message
        nc.publish(NATS_ECHO_SUBJECT, fbb.finished_data())?;

        println!("{}", full_string);

        let mut file = match OpenOptions::new().write(true).open("/dev/ttyhost") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to open /dev/ttyhost: {}", e);
                continue;
            }
        };

        // Write the string to /dev/ttyhost
        if let Err(e) = writeln!(file, "{}", full_string) {
            eprintln!("Failed to write to /dev/ttyhost: {}", e);
        }
    }
    Ok(())
}
