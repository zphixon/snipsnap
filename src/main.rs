mod auth;
mod error;
mod resource;

#[cfg(target_os = "windows")]
mod win_ui;
#[cfg(target_os = "windows")]
use win_ui as ui;

use error::MyError;

static mut CURRENT_TRACK: String = String::new();

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
            match auth::authorize() {
                Ok(new_auth) => auth = Some(new_auth),
                _ => break 'main, // errors from authorize() show up in the browser
            }
        }

        match get_currently_playing(&auth.as_ref().unwrap().access_token, file, format) {
            Some(MyError::Unauthorized) => {
                // if we're unauthorized try refreshing the access token
                match auth::refresh(&auth.unwrap().refresh_token) {
                    Ok(new_auth) => auth = Some(new_auth),

                    Err(refresh_err) => {
                        // if that didn't work open the browser and try again
                        match auth::authorize() {
                            Ok(new_auth) => auth = Some(new_auth),

                            // give up if it doesn't work
                            Err(auth_err) => {
                                ui::error(
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
                ui::error("Filesystem encountered an error.", MyError::FsError, None);
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
