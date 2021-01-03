#![feature(proc_macro_hygiene, decl_macro)]

use std::env;
use std::path::PathBuf;

use base64;
use webbrowser;
use ini::Ini;
use rocket::*;
use reqwest::{
    blocking::Client,
    header
};
use dirs::home_dir;
use serde_json::{self, Value};


const SPOTIFY_BASE_URL: &str = "https://api.spotify.com/v1";
const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const RESPONSE_TYPE: &str = "code";
const SCOPE: &str = "playlist-read-private";
const REDIRECT_URI: &str = "http://localhost:8000/auth"; 
const STATE: &str = ""; // this should be randomly generated
const GRANT_TYPE: &str = "authorization_code";


// IDs of the relevant Spotify playlists
pub const COFFEE_IN_THE_MORNING: &str = "2c5gRvQIaoMKouEo6OiTuu";
pub const SENT_TO_YOU_WITH_LOVE: &str = "2TycG938H80pPBzICl6puP";
pub const SZN21: &str = "1w1A3JJdtgafmO6IY7KwZu";
pub const SZN20: &str = "3gkUkvtdfQ6s1p8N3dTR9B";
pub const SZN19: &str = "2zWEfyf0OMwp39Xds6rYjY";
pub const SZN18: &str = "4eGeFRom9u43A04le8hCAK";
pub const BANG_YOUR_LINE: &str = "7kNphr0fgjihoAnfk0mK0K";


pub struct Track {
    pub name: String,
    pub url: String
}


fn get_credentials_file_path() -> PathBuf {
    let mut save_path = home_dir().unwrap(); // this can never fail for me so unwrap
    save_path.push(".spotify");
    save_path.push("credentials");
    save_path
}

// fn get_access_token(refresh_token: &str) {

// }

fn make_request(client: &Client, url: &str, access_token: &str) -> Value {
    let response = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .unwrap()
        .text()
        .unwrap();

    // TODO: Improve error handling
    let v: Value = serde_json::from_str(&response).unwrap();
    v
}

fn get_tracks(response: &Value) -> Vec<Track> {
    let items = response["items"].as_array().unwrap();

    let mut tracks = Vec::new();
    for item in items.iter() {
        let spotify_url = match item["track"]["external_urls"]["spotify"].as_str() {
            Some(url) => url,
            None => continue
        };

        let name = match item["track"]["name"].as_str() {
            Some(name) => name,
            None => continue
        };

        let t = Track {
            name: String::from(name),
            url: String::from(spotify_url)
        };
        tracks.push(t);
    }

    tracks
}

pub fn get_all_tracks(playlist_id: &str) -> Vec<Track> {
    // TODO: Get a new access token every invocation
    let credentials = Ini::load_from_file(get_credentials_file_path()).unwrap();
    let access_token = credentials
        .get_from(Some("default"), "access_token")
        .unwrap();

    let mut tracks_url = format!("https://api.spotify.com/v1/playlists/{playlist_id}/\
                              tracks?fields=next,items(track(name,external_urls))",
                              playlist_id=playlist_id);

    let client = Client::new();
    let mut tracks: Vec<Track> = Vec::new();

    // Paginate
    loop {
        let response = make_request(&client, &tracks_url, access_token);
        let mut t = get_tracks(&response);
        tracks.append(&mut t);

        tracks_url = match response["next"].as_str() {
            Some(url) => String::from(url),
            None => return tracks
        };
    }
}


pub fn do_auth() {
    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
    let url = format!("{auth_url}?client_id={client_id}&response_type={response_type}\
                      &redirect_uri={redirect_uri}&scope={scope}&state={state}\
                      &show_dialog={show_dialog}",
                      auth_url=SPOTIFY_AUTH_URL,
                      client_id=client_id,
                      response_type=RESPONSE_TYPE,
                      redirect_uri=REDIRECT_URI,
                      scope=SCOPE,
                      state=STATE,
                      show_dialog=false);
    webbrowser::open(&url).unwrap();
    rocket::ignite().mount("/", routes![auth]).launch();
}


#[get("/auth?<code>&<state>")]
fn auth(code: String, state: String) {
    if state == STATE {
        let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();

        let encode = format!("{client_id}:{client_secret}", client_id=client_id, client_secret=client_secret);
        let encode = base64::encode_config(encode, base64::STANDARD);
        let auth = format!("Basic {}", encode);

        let body = format!("grant_type={grant_type}&code={code}&redirect_uri={redirect_uri}",
            grant_type=GRANT_TYPE,
            code=code.trim(),
            redirect_uri=REDIRECT_URI
        );

        let client = Client::new();
        let response = client.post(SPOTIFY_TOKEN_URL)
            .header(header::CONTENT_LENGTH, body.len())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::AUTHORIZATION, auth)
            .body(body)
            .send()
            .unwrap()
            .text()
            .unwrap();
        
        let v: Value = serde_json::from_str(&response).unwrap();
        let access_token = v["access_token"]
            .as_str()
            .unwrap();
        let refresh_token = v["refresh_token"]
            .as_str()
            .unwrap();


        let save_path = get_credentials_file_path();
        let mut conf = Ini::new();
        conf.with_section(Some("default"))
            .set("access_token", access_token)
            .set("refresh_token", refresh_token);
        conf.write_to_file(save_path).unwrap();
    }
}

