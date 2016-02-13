use std::error::Error;
use std::fmt::Error as FmtError;
use std::clone::Clone;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use std::convert::From;

/**
 * Kind of store error
 */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkupErrorKind {
    // Nothing here yet
}

fn error_type_as_str(e: &MarkupErrorKind) -> &'static str {
    match e {
        _ => "",
    }
}

impl Display for MarkupErrorKind {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "{}", error_type_as_str(self)));
        Ok(())
    }

}

/**
 * Store error type
 */
#[derive(Debug)]
pub struct MarkupError {
    err_type: MarkupErrorKind,
    cause: Option<Box<Error>>,
}

impl MarkupError {

    /**
     * Build a new StoreError from an StoreErrorKind, optionally with cause
     */
    pub fn new(errtype: MarkupErrorKind, cause: Option<Box<Error>>) -> MarkupError {
        MarkupError {
            err_type: errtype,
            cause: cause,
        }
    }

    /**
     * Get the error type of this StoreError
     */
    pub fn err_type(&self) -> MarkupErrorKind {
        self.err_type.clone()
    }

}

impl Display for MarkupError {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "[{}]", error_type_as_str(&self.err_type.clone())));
        Ok(())
    }

}

impl Error for MarkupError {

    fn description(&self) -> &str {
        error_type_as_str(&self.err_type.clone())
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e)
    }

}

