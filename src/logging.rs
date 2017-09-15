use std;
use log;
use fern;

pub fn main_logger() -> fern::Dispatch {
    let level = match std::env::var("DEBUG") {
        Ok(_) => log::LogLevelFilter::Debug,
        Err(_) => log::LogLevelFilter::Info,
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
}
