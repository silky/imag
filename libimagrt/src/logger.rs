use std::io::Write;

use log::{Log, LogLevel, LogRecord, LogMetadata};

use term::color::*;
use term::stderr;
use term::StderrTerminal;

pub struct ImagLogger {
    lvl: LogLevel,
    debug_color: Color,
    info_color:  Color,
    warn_color:  Color,
    error_color: Color,
}

impl ImagLogger {

    pub fn new(lvl: LogLevel,
               debug_color: Color,
               info_color:  Color,
               warn_color:  Color,
               error_color: Color) -> ImagLogger {
        ImagLogger {
            lvl: lvl,
            debug_color: debug_color,
            info_color:  info_color,
            warn_color:  warn_color,
            error_color: error_color,
        }
    }

}

impl Log for ImagLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.lvl
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut term = stderr().unwrap();
            let fg = match record.level() {
                LogLevel::Trace => self.error_color,
                LogLevel::Error => self.error_color,
                LogLevel::Warn  => self.warn_color,
                LogLevel::Info  => self.info_color,
                LogLevel::Debug => self.debug_color,
            };
            term.fg(fg).unwrap();
            write!(term, "[imag][{: <5}]: ", record.level());

            if record.level() != LogLevel::Trace && record.level() != LogLevel::Error {
                term.fg(WHITE).unwrap();
            }

            writeln!(term, "{}", record.args());
        }
    }
}

