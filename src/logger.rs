use crate::println;
use log::{Level, Metadata, Record};
static LOGGER: Logger = Logger;

pub fn init(level: Level) {
    if let Ok(_) = log::set_logger(&LOGGER) {
        log::set_max_level(level.to_level_filter())
    }else{
        panic!("Global logger set failed!");
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}] - [{}] - {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
