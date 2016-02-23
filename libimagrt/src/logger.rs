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
            // TODO: This is just simple logging. Maybe we can enhance this lateron
            writeln!(stderr(), "[imag][{: <5}]: {}", record.level(), record.args());
        }
    }
}

