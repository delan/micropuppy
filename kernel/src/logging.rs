use core::fmt::{self, Write};

use crate::a53::pl011::Pl011RegisterBlock;

pub fn init(writer: Pl011Writer, max_level: log::LevelFilter) {
    unsafe { WRITER = Some(writer) };
    log::set_logger(&Logger).unwrap();
    log::set_max_level(max_level);
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        unsafe { WRITER.is_some() }
    }

    fn log(&self, record: &log::Record) {
        if let Some(writer) = unsafe { &mut WRITER } {
            let level = record.level();
            let file = record.file().unwrap_or("<unknown file>");
            let line = record.line().unwrap_or(0);
            let args = record.args();

            let level_style = match level {
                log::Level::Error => "\x1b[31m\x1b[1m",
                log::Level::Warn => "\x1b[33m",
                log::Level::Info => "\x1b[32m",
                log::Level::Debug => "\x1b[34m",
                log::Level::Trace => "\x1b[36m",
            };
            let sgr0 = "\x1b[0m";

            writeln!(
                writer,
                "[{level_style}{level:<5}{sgr0} {file}:{line}] {args}"
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}

pub static mut WRITER: Option<Pl011Writer> = None;

pub struct Pl011Writer(*mut Pl011RegisterBlock);

impl Pl011Writer {
    pub fn new(base_address: *const u8) -> Self {
        Self(base_address as *mut Pl011RegisterBlock)
    }
}

impl fmt::Write for Pl011Writer {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let uart = unsafe { &*self.0 };
        for byte in s.bytes() {
            uart.dr.write_initial(|w| w.data(byte));
        }

        Ok(())
    }
}
