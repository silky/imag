use std::error::Error;
use std::io::Write;

use term::color::*;
use term::stderr;
use term::StderrTerminal;

/// Print an Error type and its cause recursively
///
/// The error is printed with "Error NNNN :" as prefix, where "NNNN" is a number which increases
/// which each recursion into the errors cause. The error description is used to visualize what
/// failed and if there is a cause "-- caused by:" is appended, and the cause is printed on the next
/// line.
///
/// Example output:
///
/// ```ignore
/// Error    1 : Some error -- caused by:
/// Error    2 : Some other error -- caused by:
/// Error    3 : Yet another Error -- caused by:
/// ...
///
/// Error <NNNN> : <Error description>
/// ```
pub fn trace_error(e: &Error) {
    let mut term = stderr().unwrap();
    print_trace_maxdepth(&mut term, count_error_causes(e), e, ::std::u64::MAX);
    write!(term, "");
}

/// Print an Error type and its cause recursively, but only `max` levels
///
/// Output is the same as for `trace_error()`, though there are only `max` levels printed.
pub fn trace_error_maxdepth(e: &Error, max: u64) {
    let n = count_error_causes(e);
    let mut term = stderr().unwrap();
    term.fg(BRIGHT_RED);
    write!(term, "{}/{} Levels of errors will be printed", (if max > n { n } else { max }), n);
    print_trace_maxdepth(&mut term, n, e, max);
    write!(term, "");
}

/// Print an Error type and its cause recursively with the debug!() macro
///
/// Output is the same as for `trace_error()`.
pub fn trace_error_dbg(e: &Error) {
    print_trace_dbg(0, e);
}

/// Helper function for `trace_error()` and `trace_error_maxdepth()`.
///
/// Returns the cause of the last processed error in the recursion, so `None` if all errors where
/// processed.
fn print_trace_maxdepth<'a, 'b>(term: &'a mut Box<StderrTerminal>, idx: u64, e: &'b Error, max: u64) -> Option<&'b Error> {
    if e.cause().is_some() && idx > 0 {
        print_trace_maxdepth(term, idx - 1, e.cause().unwrap(), max);
        term.fg(WHITE);
        write!(term, " -- caused:");
    }
    term.fg(BRIGHT_RED);
    write!(term, "Error {:>4} :", idx);
    term.fg(WHITE);
    write!(term, "{}", e.description());
    e.cause()
}

/// Count errors in Error::cause() recursively
fn count_error_causes(e: &Error) -> u64 {
    1 + if e.cause().is_some() { count_error_causes(e.cause().unwrap()) } else { 0 }
}

fn print_trace_dbg(idx: u64, e: &Error) {
    debug!("Error {:>4} : {}", idx, e.description());
    if e.cause().is_some() {
        debug!(" -- caused by:");
        print_trace_dbg(idx + 1, e.cause().unwrap());
    }
}

