use chrono::{Datelike, Local, Timelike};
use env_logger::Env;
use std::io::Write;

const FULL_DEBUG_INFO: bool = false;

pub fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            let now = Local::now();
            if FULL_DEBUG_INFO {
                writeln!(
                    buf,
                    "[{}/{:02}/{:02} {:02}:{:02}:{:02}.{:03} {} {}] {}",
                    now.year(),
                    now.month(),
                    now.day(),
                    now.hour(),
                    now.minute(),
                    now.second(),
                    now.timestamp_subsec_millis(),
                    record.level(),
                    record.module_path().unwrap_or("unknown"),
                    record.args()
                )
            } else {
                writeln!(
                    buf,
                    "[{}] {}",
                    record.module_path().unwrap_or("unknown"),
                    record.args()
                )
            }
        })
        .init();
}
