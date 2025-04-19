use std::io::Write;

use env_logger::fmt::Formatter;
use log::Record;

fn main() {
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::Logger::from_default_env()
            .with_format(Box::new(|buf: &mut Formatter, record: &Record<'_>| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            }))
            .try_init()
            .unwrap(),
        _ => env_logger::init(),
    };
}
