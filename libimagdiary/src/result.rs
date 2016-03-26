use std::result::Result as RResult;

use error::DiaryError;

pub type Result<T> = RResult<T, DiaryError>;
