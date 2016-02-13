use std::result::Result as RResult;

use error::MarkupError;

pub type Result<T> = RResult<T, MarkupError>;

