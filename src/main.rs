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

    // send user to the browser
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(250));
        match webbrowser::open("http://localhost:44554/snipsnap") {
            Ok(_) => {}
            Err(e) => crate::error::error_msg(e.into()),
        };
    });

    let _: Result<(), ()> = auth::authorize()
        .map(|mut auth| loop {
            let _: Result<(), ()> = get_currently_playing(&auth.access_token)
                .map(|track| {
                    std::fs::write(file, track)
                        .map_err(|e| {
                            error::error_msg(e.into());
                            std::process::exit(1);
                        })
                        .unwrap();
                })
                .map_err(|e| match e {
                    MyError::Unauthorized => {
                        auth = auth::refresh(&auth.refresh_token)
                            .map_err(|e| {
                                error::error_msg(e);
                                std::process::exit(1);
                            })
                            .unwrap();
                    }
                    e => {
                        error::error_msg(e);
                        std::process::exit(1);
                    }
                });
            std::thread::sleep(std::time::Duration::from_secs(10));
        })
        .map_err(|e| {
            error::error_msg(e);
            std::process::exit(1);
        });
}

fn get_currently_playing(access_token: &str) -> Result<String, MyError> {
    let response = ureq::get("https://api.spotify.com/v1/me/player/currently-playing")
        .set("Authorization", &format!("Bearer {}", access_token))
        .send_string("");

    if response.status() == 204 {
        // 204: No Content (not currently playing anything)
        return Ok(String::new());
    } else if response.status() == 401 {
        // 401: Unauthorized (need to refresh or authorize)
        return Err(MyError::Unauthorized);
    }

    let json = response.into_json()?;

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

    return if unsafe { &CURRENT_TRACK } == &track {
        Ok(String::new())
    } else {
        Ok(track)
    };
}
