use std::error::Error;
use std::fmt::Error as FmtError;
use std::clone::Clone;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinkErrorKind {
    EntryHeaderReadError,
    EntryHeaderWriteError,
    ExistingLinkTypeWrong,
    LinkTargetDoesNotExist,
    InternalConversionError,
    InvalidUri,
    StoreReadError,
}

fn link_error_type_as_str(e: &LinkErrorKind) -> &'static str {
    match e {
        &LinkErrorKind::EntryHeaderReadError
            => "Error while reading an entry header",

        &LinkErrorKind::EntryHeaderWriteError
            => "Error while writing an entry header",

        &LinkErrorKind::ExistingLinkTypeWrong
            => "Existing link entry has wrong type",

        &LinkErrorKind::LinkTargetDoesNotExist
            => "Link target does not exist in the store",

        &LinkErrorKind::InternalConversionError
            => "Error while converting values internally",

        &LinkErrorKind::InvalidUri
            => "URI is not valid",

        &LinkErrorKind::StoreReadError
            => "Store read error",
    }
}

impl Display for LinkErrorKind {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "{}", link_error_type_as_str(self)));
        Ok(())
    }

}

#[derive(Debug)]
pub struct LinkError {
    kind: LinkErrorKind,
    cause: Option<Box<Error>>,
}

impl LinkError {

    pub fn new(errtype: LinkErrorKind, cause: Option<Box<Error>>) -> LinkError {
        LinkError {
            kind: errtype,
            cause: cause,
        }
    }

}

impl Display for LinkError {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "[{}]", link_error_type_as_str(&self.kind)));
        Ok(())
    }

}

impl Error for LinkError {

    fn description(&self) -> &str {
        link_error_type_as_str(&self.kind)
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e)
    }

}
