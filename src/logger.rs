use crate::println;
use log::{Level, Metadata, Record};

static LOGGER: Logger = Logger;

pub fn init(level: Level) {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level.to_level_filter())
    } else {
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
            match record.level() {
                Level::Error => println!("\x1b[31m{}\x1b[0m", record.args()),
                Level::Warn => println!("\x1b[33m{}\x1b[0m", record.args()),
                Level::Info => println!("\x1b[32m{}\x1b[0m", record.args()),
                Level::Debug => println!("\x1b[34m{}\x1b[0m", record.args()),
                Level::Trace => println!("\x1b[35m{}\x1b[0m", record.args()),
            }
        }
    }

    fn flush(&self) {}
}

unsafe impl Sync for Logger {}
