use crate::resource::*;

pub fn error_msg(e: MyError) {
    msgbox::create("SnipSnap", &format!("{}", e), msgbox::IconType::Error).unwrap();
}

#[derive(Debug, Clone)]
pub enum MyError {
    CrossSiteRequestForgery,
    TokenRequestError(String),
    AccessDenied,
    RefreshFailed,
    Unauthorized,
    InvalidResponseJson(String),
    Io(String),
    Unknown(String),
}

use std::fmt;
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MyError::*;
        match self {
            CrossSiteRequestForgery => write!(f, "{}", CROSS_SITE_REQUEST_FORGERY_MSG),
            TokenRequestError(e) => {
                write!(f, "{}\n{}\n\n{}", TOKEN_REQUEST_ERROR_MSG, BUG_REPORT, e)
            }
            AccessDenied => write!(f, "{}", ACCESS_DENIED_MSG),
            RefreshFailed => write!(f, "{}", REFRESH_FAILED_MSG),
            Unauthorized => write!(f, "{}", UNAUTHORIZED_MSG),
            InvalidResponseJson(e) => {
                write!(f, "{}\n{}\n\n{}", INVALID_RESPONSE_JSON_MSG, BUG_REPORT, e)
            }
            Io(e) => write!(f, "{}\n{}", IO_ERROR_MSG, e),
            Unknown(e) => write!(f, "{}\n{}\n\n{}", UNKNOWN_MSG, BUG_REPORT, e),
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::Io(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for MyError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        MyError::Io(e.to_string())
    }
}
