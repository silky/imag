use clap::Arg;

use std::result::Result as RResult;

use filter::Filter;

pub fn generate_filter_arg(short: Option<&str>) -> Arg {
    let arg = Arg::with_name("filter")
        .long("filter")
        .takes_value(true)
        .help("Content for the Entry from commandline");

    if short.is_some() {
        arg.short(short.unwrap())
    } else {
        arg
    }
}

pub mod error {
    use std::error::Error;
    use std::fmt::Error as FmtError;
    use std::clone::Clone;
    use std::fmt::{Display, Formatter};

    /**
     * Kind of store error
     */
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum FilterCompilerErrorKind {
        BackendError,
        NoCommandlineCall,
            // maybe more
    }

    fn fce_error_type_as_str(e: &FilterCompilerErrorKind) -> &'static str {
        match e {
            &FilterCompilerErrorKind::BackendError      => "Backend Error",
            &FilterCompilerErrorKind::NoCommandlineCall => "No commandline call",
        }
    }

    impl Display for FilterCompilerErrorKind {

        fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
            try!(write!(fmt, "{}", fce_error_type_as_str(self)));
            Ok(())
        }

    }

    #[derive(Debug)]
    pub struct FilterCompilerError {
        err_type: FilterCompilerErrorKind,
        cause: Option<Box<Error>>,
    }

    impl FilterCompilerError {

        /**
         * Build a new FilterCompilerError from an FilterCompilerErrorKind, optionally with cause
         */
        pub fn new(errtype: FilterCompilerErrorKind, cause: Option<Box<Error>>)
            -> FilterCompilerError
            {
                FilterCompilerError {
                    err_type: errtype,
                    cause: cause,
                }
            }

        /**
         * Get the error type of this FilterCompilerError
         */
        pub fn err_type(&self) -> FilterCompilerErrorKind {
            self.err_type.clone()
        }

    }

    impl Display for FilterCompilerError {

        fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
            try!(write!(fmt, "[{}]", fce_error_type_as_str(&self.err_type.clone())));
            Ok(())
        }

    }

    impl Error for FilterCompilerError {

        fn description(&self) -> &str {
            fce_error_type_as_str(&self.err_type.clone())
        }

        fn cause(&self) -> Option<&Error> {
            self.cause.as_ref().map(|e| &**e)
        }

    }
}

use self::error::FilterCompilerError;

pub type Result<T> = RResult<T, FilterCompilerError>;

pub fn compile(source: &str) -> Result<Box<Filter>> {
    unimplemented!()
}

