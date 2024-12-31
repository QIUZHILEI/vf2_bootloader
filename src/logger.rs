use crate::println;
use log::{Level, Metadata, Record};
static LOGGER: Logger = Logger;

pub fn init(level: Level) {
    if log::set_logger(&LOGGER).is_ok() {
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
                "[{}] {}",
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
