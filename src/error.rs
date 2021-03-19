#[derive(Debug, Copy, Clone)]
pub enum MyError {
    CrossSiteRequestForgery,
    TokenRequestError,
    AccessDenied,
    RefreshFailed,
    Unauthorized,
    InvalidResponseJson,
    FsError,
    Unknown,
}
