use std::result::Result as RResult;

use libimagstore::store::FileLockEntry;

pub mod abbrev;
pub mod plain;
pub mod table;

pub mod error {
    use std::error::Error;
    use std::fmt::Error as FmtError;
    use std::clone::Clone;
    use std::fmt::{Debug, Display, Formatter};
    use std::fmt;
    use std::convert::From;

    /**
     * Kind of entry_printer error
     */
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum EntryPrinterErrorKind {
            // maybe more
    }

    fn entry_printer_error_type_as_str(e: &EntryPrinterErrorKind) -> &'static str {
        match e {
            _ => "",
        }
    }

    impl Display for EntryPrinterErrorKind {

        fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
            try!(write!(fmt, "{}", entry_printer_error_type_as_str(self)));
            Ok(())
        }

    }

    /**
     * EntryPrinter error type
     */
    #[derive(Debug)]
    pub struct EntryPrinterError {
        err_type: EntryPrinterErrorKind,
        cause: Option<Box<Error>>,
    }

    impl EntryPrinterError {

        /**
         * Build a new EntryPrinterError from an EntryPrinterErrorKind, optionally with cause
         */
        pub fn new(errtype: EntryPrinterErrorKind, cause: Option<Box<Error>>)
            -> EntryPrinterError
            {
                EntryPrinterError {
                    err_type: errtype,
                    cause: cause,
                }
            }

        /**
         * Get the error type of this EntryPrinterError
         */
        pub fn err_type(&self) -> EntryPrinterErrorKind {
            self.err_type.clone()
        }

    }

    impl Display for EntryPrinterError {

        fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
            try!(write!(fmt, "[{}]", entry_printer_error_type_as_str(&self.err_type.clone())));
            Ok(())
        }

    }

    impl Error for EntryPrinterError {

        fn description(&self) -> &str {
            entry_printer_error_type_as_str(&self.err_type.clone())
        }

        fn cause(&self) -> Option<&Error> {
            self.cause.as_ref().map(|e| &**e)
        }

    }

}

use self::error::EntryPrinterError;

pub type Result<T> = RResult<T, EntryPrinterError>;

pub trait EntryPrinter {

    fn print_entry(&self, e: &FileLockEntry) -> Result<()>;

    fn print_entries(&self, es: Vec<&FileLockEntry>) -> Result<()> {
        es.iter().map(|e| self.print_entry(e)).filter(|res| res.is_err()).next().unwrap_or(Ok(()))
    }

}

