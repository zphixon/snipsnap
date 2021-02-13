use rand::Rng;
use sha2::Digest;
use tiny_http::{Header, Method, Response, Server};
use url::Url;

use std::str::FromStr;

const SPOTIFY_TOKEN_URL: &'static str = "https://accounts.spotify.com/api/token";
const SPOTIFY_AUTH_URL: &'static str = "https://accounts.spotify.com/authorize?response_type=code";
const URL: &'static str = "http://localhost:44554/snipsnap";
const CLIENT_ID: &'static str = "9b4c0a0ae4a54117b5201c2beb15361f";

#[derive(Debug, Copy, Clone)]
enum MyError {
    CrossSiteRequestForgery,
    TokenRequestError,
    AccessDenied,
    RefreshFailed,
    Unauthorized,
    InvalidResponseJson,
    FsError,
    Unknown,
}

struct Auth {
    pub access_token: String,
    pub refresh_token: String,
}

static mut CURRENT_TRACK: String = String::new();

macro_rules! bug_report {
    () => {
        "Looks like something went wrong. Please file a bug report at https://github.com/zphixon/snipsnap\n\n{:#?}"
    }
}

fn error(msg: &str, e1: MyError, e2: Option<MyError>) {
    msgbox::create(
        "SnipSnap",
        &format!("{}\n\n{:?}, {:?}", msg, e1, e2),
        msgbox::IconType::Error,
    )
    .unwrap();
}

fn main() {
    // TODO: add customization
    // * port number
    // * filename
    // * format

    let file = "track.txt";
    let format = "";

    let mut auth = None;

    'main: loop {
        if auth.is_none() {
            match authorize() {
                Ok(new_auth) => auth = Some(new_auth),
                _ => break 'main, // errors from authorize() show up in the browser
            }
        }

        match get_currently_playing(&auth.as_ref().unwrap().access_token, file, format) {
            Some(MyError::Unauthorized) => {
                // if we're unauthorized try refreshing the access token
                match refresh(&auth.unwrap().refresh_token) {
                    Ok(new_auth) => auth = Some(new_auth),

                    Err(refresh_err) => {
                        // if that didn't work open the browser and try again
                        match authorize() {
                            Ok(new_auth) => auth = Some(new_auth),

                            // give up if it doesn't work
                            Err(auth_err) => {
                                error(
                                    "Please check your network connection or Spotify account.",
                                    refresh_err,
                                    Some(auth_err),
                                );

                                break 'main;
                            }
                        }
                    }
                }
            }

            Some(MyError::FsError) => {
                error("Filesystem encountered an error.", MyError::FsError, None);
            }

            // get_currently_playing does not return any other error type
            Some(_) => unreachable!(),

            None => {}
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn get_currently_playing(access_token: &str, path: &str, _format: &str) -> Option<MyError> {
    let response = ureq::get("https://api.spotify.com/v1/me/player/currently-playing")
        .set("Authorization", &format!("Bearer {}", access_token))
        .send_string("");

    if response.status() == 204 {
        // 204: No Content (not currently playing anything)
        return std::fs::write(path, "").map_err(|_| MyError::FsError).err();
    } else if response.status() == 401 {
        // 401: Unauthorized (need to refresh or authorize)
        return Some(MyError::Unauthorized);
    }

    let json = response.into_json().unwrap();

    // collect artists
    let mut artist = String::new();
    for (i, artist_json) in json["item"]["artists"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        artist.push_str(artist_json["name"].as_str().unwrap());
        if i + 1 != json["item"]["artists"].as_array().unwrap().len() {
            artist.push_str(", ");
        }
    }

    let track = format!("{} - {}", json["item"]["name"].as_str().unwrap(), artist);

    if unsafe { &CURRENT_TRACK } == &track {
        None
    } else {
        std::fs::write(path, track)
            .map_err(|_| MyError::FsError)
            .err()
    }
}

fn refresh(refresh_token: &str) -> Result<Auth, MyError> {
    let refresh_url = Url::parse_with_params(
        SPOTIFY_TOKEN_URL,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", CLIENT_ID),
        ],
    )
    .unwrap();

    match ureq::post(refresh_url.as_str())
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string("")
        .into_json()
    {
        Ok(new_auth) => {
            if new_auth["error"].is_null() {
                Ok(Auth {
                    access_token: new_auth["access_token"].as_str().unwrap().to_owned(),
                    refresh_token: new_auth["refresh_token"].as_str().unwrap().to_owned(),
                })
            } else {
                Err(MyError::RefreshFailed)
            }
        }

        _ => Err(MyError::InvalidResponseJson),
    }
}

// authorize using PKCE
fn authorize() -> Result<Auth, MyError> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    // generate some state, optional but recommended
    let state: String = (0..10)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    // generate a verifier
    let verifier: String = (0..124)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    // hash the verifier
    let mut hasher = sha2::Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();

    // create the challenge
    let challenge = base64::encode_config(&result[..], base64::URL_SAFE_NO_PAD);

    // url which the user will be directed to in order to start auth
    let auth_uri = Url::parse_with_params(
        SPOTIFY_AUTH_URL,
        &[
            ("client_id", CLIENT_ID),
            ("redirect_uri", URL),
            ("code_challenge_method", "S256"),
            ("code_challenge", &challenge),
            ("scope", "user-read-currently-playing"),
            ("state", &state),
        ],
    )
    .unwrap();

    // start up server
    let server = Server::http("0.0.0.0:44554").expect("server!");

    // send user to the browser
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(250));
        webbrowser::open("http://localhost:44554/snipsnap").unwrap();
    });

    for request in server.incoming_requests() {
        match request.method() {
            // browser page loads
            Method::Get if request.url() == "/snipsnap" => {
                // immediately redirect user to spotify authorization page
                let response = Response::from_string("redirecting...")
                    .with_status_code(303)
                    .with_header(
                        Header::from_str(&format!("Location: {}", auth_uri.as_str())).unwrap(),
                    );

                request.respond(response).unwrap();
            }

            // second stage of initial app authorization, after user allows/denies access
            Method::Get if request.url().starts_with("/snipsnap?") => {
                // use a full url because apparently you can't do just a path?
                let url = Url::parse(&format!("http://localhost{}", request.url())).unwrap();
                let queries: Vec<(String, String)> = url
                    .query_pairs()
                    .map(|(s1, s2)| (s1.to_string(), s2.to_string()))
                    .collect();

                // check that the state is the same
                if let Some((_, response_state)) = queries.iter().find(|(name, _)| name == "state")
                {
                    if response_state != &state {
                        request
                            .respond(Response::from_string(
                                "Cross-site request forgery detected. Exiting auth flow. Your network may be compromised or incorrectly configured.",
                            ))
                            .unwrap();
                        return Err(MyError::CrossSiteRequestForgery);
                    }
                }

                // user denies access or some other error occurs
                if let Some((_, error)) = queries.iter().find(|(name, _)| name == "error") {
                    if error == "access_denied" {
                        request
                            .respond(Response::from_string("User denied access."))
                            .unwrap();
                        return Err(MyError::AccessDenied);
                    } else {
                        let error = format!(bug_report!(), error);
                        request.respond(Response::from_string(error)).unwrap();
                        return Err(MyError::Unknown);
                    }
                }

                // get the authorization code
                let (_, code) = queries.iter().find(|(name, _)| name == "code").unwrap();

                // url to request an access token
                let token_url = Url::parse_with_params(
                    SPOTIFY_TOKEN_URL,
                    &[
                        ("client_id", CLIENT_ID),
                        ("grant_type", "authorization_code"),
                        ("code", code.as_ref()),
                        ("redirect_uri", URL),
                        ("code_verifier", &verifier),
                    ],
                )
                .unwrap();

                // request an access token
                match ureq::post(token_url.as_str())
                    .set("Content-Type", "application/x-www-form-urlencoded") // <-- this is important
                    .send_string("") // <-- this also, it sent the Content-Length header which is required
                    .into_json()
                {
                    Ok(response) => {
                        if response["error"].is_null() {
                            // woohoo
                            request
                                .respond(Response::from_string(
                                    "SnipSnap is set up! You can now close this window.",
                                ))
                                .unwrap();

                            return Ok(Auth {
                                access_token: response["access_token"].as_str().unwrap().to_owned(),
                                refresh_token: response["refresh_token"]
                                    .as_str()
                                    .unwrap()
                                    .to_owned(),
                            });
                        } else {
                            // something in the request went wrong
                            let error = format!(bug_report!(), response);
                            request.respond(Response::from_string(error)).unwrap();
                            return Err(MyError::TokenRequestError);
                        }
                    }

                    Err(error) => {
                        // the json was bad, this is probably spotify's fault
                        let error = format!(bug_report!(), error);
                        request.respond(Response::from_string(error)).unwrap();
                        return Err(MyError::InvalidResponseJson);
                    }
                }
            }

            _ => request
                .respond(Response::from_string(
                    "Hi, glad you're enjoying SnipSnap enough to poke around ♥",
                ))
                .unwrap(),
        }
    }

    unreachable!();
}
