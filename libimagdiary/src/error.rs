use std::error::Error;
use std::fmt::Error as FmtError;
use std::clone::Clone;
use std::fmt::{Display, Formatter};

/**
 * Kind of error
 */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiaryErrorKind {
    StoreWriteError,
    StoreReadError,
    CannotFindDiary,
    CannotCreateNote,
    DiaryEditError,
    // Nothing here yet
}

fn note_error_type_as_str(e: &DiaryErrorKind) -> &'static str {
    match e {
        &DiaryErrorKind::StoreWriteError  => "Error writing store",
        &DiaryErrorKind::StoreReadError   => "Error reading store",
        &DiaryErrorKind::CannotFindDiary  => "Cannot find diary",
        &DiaryErrorKind::CannotCreateNote => "Cannot create Note object for diary entry",
        &DiaryErrorKind::DiaryEditError   => "Cannot edit diary entry",
    }
}

impl Display for DiaryErrorKind {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "{}", note_error_type_as_str(self)));
        Ok(())
    }

}

/**
 * Store error type
 */
#[derive(Debug)]
pub struct DiaryError {
    err_type: DiaryErrorKind,
    cause: Option<Box<Error>>,
}

impl DiaryError {

    /**
     * Build a new DiaryError from an DiaryErrorKind, optionally with cause
     */
    pub fn new(errtype: DiaryErrorKind, cause: Option<Box<Error>>) -> DiaryError {
        DiaryError {
            err_type: errtype,
            cause: cause,
        }
    }

    /**
     * Get the error type of this DiaryError
     */
    pub fn err_type(&self) -> DiaryErrorKind {
        self.err_type.clone()
    }

}

impl Display for DiaryError {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "[{}]", note_error_type_as_str(&self.err_type.clone())));
        Ok(())
    }

}

impl Error for DiaryError {

    fn description(&self) -> &str {
        note_error_type_as_str(&self.err_type.clone())
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e)
    }

}


