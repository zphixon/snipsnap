pub const SPOTIFY_TOKEN_URL: &'static str = "https://accounts.spotify.com/api/token";
pub const SPOTIFY_AUTH_URL: &'static str =
    "https://accounts.spotify.com/authorize?response_type=code";
pub const LOCAL_CALLBACK_URL: &'static str = "http://localhost:44554/snipsnap";
pub const CLIENT_ID: &'static str = "9b4c0a0ae4a54117b5201c2beb15361f";

#[macro_export]
macro_rules! bug_report {
    () => {
        "Looks like something went wrong. Please file a bug report at https://github.com/zphixon/snipsnap\n\n{:#?}"
    }
}
