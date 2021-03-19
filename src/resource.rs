pub const SPOTIFY_TOKEN_URL: &'static str = "https://accounts.spotify.com/api/token";
pub const SPOTIFY_AUTH_URL: &'static str =
    "https://accounts.spotify.com/authorize?response_type=code";
pub const LOCAL_CALLBACK_URL: &'static str = "http://localhost:44554/snipsnap";
pub const CLIENT_ID: &'static str = "9b4c0a0ae4a54117b5201c2beb15361f";

pub const REDIRECTING: &'static str = "Redirecting...";
pub const DENIED_ACCESS: &'static str = "User denied access.";
pub const SET_UP: &'static str = "SnipSnap is set up! You can now close this window.";
pub const SNEAKY_WINK: &'static str = "Hi, glad you're enjoying SnipSnap enough to poke around ♥";

pub const BUG_REPORT: &'static str = "Looks like something went wrong. Please file a bug report at https://github.com/zphixon/snipsnap";
pub const CROSS_SITE_REQUEST_FORGERY_MSG: &'static str = "Cross-site request forgery detected. Exiting auth flow. Your network may be compromised or incorrectly configured.";
pub const TOKEN_REQUEST_ERROR_MSG: &'static str = "Could not request the auth token.";
pub const ACCESS_DENIED_MSG: &'static str = "User denied access.";
pub const REFRESH_FAILED_MSG: &'static str = "Auth key refresh failed. Please relaunch SnipSnap.";
pub const UNAUTHORIZED_MSG: &'static str = "Unauthorized.";
pub const INVALID_RESPONSE_JSON_MSG: &'static str = "Invalid response json.";
pub const IO_ERROR_MSG: &'static str = "IO error.";
pub const UNKNOWN_MSG: &'static str = "Unknown other error.";
