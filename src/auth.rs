use rand::Rng;
use sha2::Digest;
use tiny_http::{Header, Method, Response, Server};
use url::Url;

use std::str::FromStr;

use crate::{error::MyError, resource::*};

pub struct Auth {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn refresh(refresh_token: &str) -> Result<Auth, MyError> {
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

        Err(e) => Err(MyError::InvalidResponseJson(e.to_string())),
    }
}

// authorize using PKCE
pub fn authorize() -> Result<Auth, MyError> {
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
            ("redirect_uri", LOCAL_CALLBACK_URL),
            ("code_challenge_method", "S256"),
            ("code_challenge", &challenge),
            ("scope", "user-read-currently-playing"),
            ("state", &state),
        ],
    )
    .unwrap();

    // start up server
    let server = Server::http("0.0.0.0:44554")?;

    for request in server.incoming_requests() {
        match request.method() {
            // browser page loads
            Method::Get if request.url() == "/snipsnap" => {
                // immediately redirect user to spotify authorization page
                let response = Response::from_string(REDIRECTING)
                    .with_status_code(303)
                    .with_header(
                        Header::from_str(&format!("Location: {}", auth_uri.as_str())).unwrap(),
                    );

                request.respond(response)?;
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
                        request.respond(Response::from_string(""))?;
                        return Err(MyError::CrossSiteRequestForgery);
                    }
                }

                // user denies access or some other error occurs
                if let Some((_, error)) = queries.iter().find(|(name, _)| name == "error") {
                    if error == "access_denied" {
                        request.respond(Response::from_string(""))?;
                        return Err(MyError::AccessDenied);
                    } else {
                        request.respond(Response::from_string(error.to_string()))?;
                        return Err(MyError::Unknown(error.to_string()));
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
                        ("redirect_uri", LOCAL_CALLBACK_URL),
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
                            request.respond(Response::from_string(SET_UP))?;

                            return Ok(Auth {
                                access_token: response["access_token"].as_str().unwrap().to_owned(),
                                refresh_token: response["refresh_token"]
                                    .as_str()
                                    .unwrap()
                                    .to_owned(),
                            });
                        } else {
                            // something in the request went wrong
                            request.respond(Response::from_string(response.to_string()))?;
                            return Err(MyError::TokenRequestError(response.to_string()));
                        }
                    }

                    Err(error) => {
                        // the json was bad, this is probably spotify's fault
                        request.respond(Response::from_string(error.to_string()))?;
                        return Err(MyError::InvalidResponseJson(error.to_string()));
                    }
                }
            }

            _ => request.respond(Response::from_string(SNEAKY_WINK))?,
        }
    }

    unreachable!();
}
