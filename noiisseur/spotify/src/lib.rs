#![feature(proc_macro_hygiene, decl_macro)]

use std::env;
use std::path::PathBuf;

use base64;
use csrf::{CsrfProtection, HmacCsrfProtection};
use dirs::home_dir;
use exitcode;
use form_urlencoded;
use hmac::{Hmac, Mac, NewMac};
use ini::Ini;
use lazy_static::{__Deref, lazy_static};
use reqwest::{blocking::Client, header};
use ring::rand::{self, SecureRandom};
use rocket::*;
use serde::Deserialize;
use serde_json;
use sha2::Sha256;
use webbrowser;

// Constants
const SPOTIFY_BASE_URL: &str = "https://api.spotify.com/v1";
const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const RESPONSE_TYPE: &str = "code";
const SCOPE: &str = "playlist-read-private";
const REDIRECT_URI: &str = "http://localhost:8000/auth";
lazy_static! {
    static ref SPOTIFY_CLIENT_ID: String =
        env::var("SPOTIFY_CLIENT_ID").expect("Missing env variable: SPOTIFY_CLIENT_ID");
    static ref SPOTIFY_CLIENT_SECRET: String =
        env::var("SPOTIFY_CLIENT_SECRET").expect("Missing env variable: SPOTIFY_CLIENT_SECRET");
    static ref STATE: String = {
        let mut bytes = [0; 32];
        let rng = rand::SystemRandom::new();
        rng.fill(&mut bytes).unwrap();

        let hmac_key = Hmac::<Sha256>::new_varkey(&bytes)
            .unwrap()
            .finalize()
            .into_bytes();

        let hmac = HmacCsrfProtection::from_key(hmac_key.into());
        let mut bytes = [0; 64];
        rng.fill(&mut bytes).unwrap();
        let token = hmac.generate_token(&bytes).unwrap().b64_url_string();
        token
    };
    static ref CREDENTIALS_FILE: PathBuf = {
        let mut save_path = home_dir().unwrap();
        save_path.push(".spotify");
        save_path.push("credentials");
        save_path
    };
}

#[derive(Deserialize)]
struct ExternalUrl {
    #[serde(rename = "spotify")]
    url: String,
}

#[derive(Deserialize)]
pub struct SpotifyTrackInner {
    #[serde(rename = "id")]
    pub spotify_id: String,
    pub name: String,
    #[serde(rename = "external_urls")]
    url: ExternalUrl,
}
#[derive(Deserialize)]
pub struct SpotifyTrack {
    #[serde(rename = "track")]
    pub track: SpotifyTrackInner
}

impl SpotifyTrack {
    pub fn spotify_id(&self) -> String {
        self.track.spotify_id.clone()
    }

    pub fn name(&self) -> String {
        self.track.name.clone()
    }

    pub fn url(&self) -> String {
        // Need to figure out how to restructure my coe
        // to remove this
        self.track.url.url.clone()
    }
}

// A Spotify track page is on object representing a paginated
// response containing Spotify tracks
#[derive(Deserialize)]
struct SpotifyTrackPage {
    #[serde(rename = "items")]
    tracks: Vec<SpotifyTrack>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct SpotifyAccessAuth<'a> {
    pub access_token: &'a str,
    pub refresh_token: &'a str,
    pub scope: &'a str,
}

#[derive(Deserialize)]
struct SpotifyRefreshAuth<'a> {
    access_token: &'a str,
}

pub fn refresh_access_token() -> String {
    let mut credentials = Ini::load_from_file(CREDENTIALS_FILE.deref()).unwrap();
    let refresh_token = credentials
        .get_from(Some("default"), "refresh_token")
        .unwrap();

    let client = Client::new();
    let body = form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "refresh_token")
        .append_pair("refresh_token", refresh_token)
        .finish();

    let auth_credentials = {
        let credentials = format!(
            "{client_id}:{client_secret}",
            client_id = *SPOTIFY_CLIENT_ID,
            client_secret = *SPOTIFY_CLIENT_SECRET
        );
        let credentials = format!(
            "Basic {}",
            base64::encode_config(credentials, base64::STANDARD)
        );
        credentials
    };

    let response = client
        .post(SPOTIFY_TOKEN_URL)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::AUTHORIZATION, auth_credentials)
        .body(body)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let refresh_auth: SpotifyRefreshAuth = serde_json::from_str(&response).unwrap();

    credentials
        .with_section(Some("default"))
        .set("access_token", refresh_auth.access_token);
    credentials.write_to_file(CREDENTIALS_FILE.deref()).unwrap();

    refresh_auth.access_token.to_string()
}

pub fn get_tracks(playlist_id: &str, access_token: &str) -> Vec<SpotifyTrack> {
    let mut tracks_url = format!(
        "https://api.spotify.com/v1/playlists/{playlist_id}/\
        tracks?fields=next,items(track(id,name,external_urls))",
        playlist_id = playlist_id
    );

    let client = Client::new();
    let mut tracks: Vec<SpotifyTrack> = Vec::new();

    // Paginate
    loop {
        let mut response = {
            let response = client
                .get(&tracks_url)
                .bearer_auth(access_token)
                .send()
                .unwrap()
                .text()
                .unwrap();

            // TODO: Improve error handling
            let response: SpotifyTrackPage = serde_json::from_str(&response).unwrap();
            response
        };
        tracks.append(&mut response.tracks);

        tracks_url = match response.next {
            Some(url) => String::from(url),
            None => break,
        };
    }

    tracks
}

pub fn authenticate() {
    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
    let url = {
        let params = form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &client_id)
            .append_pair("response_type", RESPONSE_TYPE)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", SCOPE)
            .append_pair("state", &STATE)
            .append_pair("show_dialog", "false")
            .finish();
            
        let url = format!(
            "{auth_url}?{params}",
            auth_url = SPOTIFY_AUTH_URL,
            params = params
        );
        url
    };

    webbrowser::open(&url).unwrap();
    rocket::ignite().mount("/", routes![_authenticate]).launch();
}

#[get("/auth?<code>&<state>")]
fn _authenticate(code: String, state: String) {
    if state == *STATE {
        let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();

        let encode = format!(
            "{client_id}:{client_secret}",
            client_id = client_id,
            client_secret = client_secret
        );
        let encode = base64::encode_config(encode, base64::STANDARD);
        let auth = format!("Basic {}", encode);

        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code.trim())
            .append_pair("redirect_uri", REDIRECT_URI)
            .finish();

        let client = Client::new();
        let response = client
            .post(SPOTIFY_TOKEN_URL)
            .header(header::CONTENT_LENGTH, body.len())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::AUTHORIZATION, auth)
            .body(body)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let access_auth: SpotifyAccessAuth = serde_json::from_str(&response).unwrap();

        let mut conf = Ini::new();
        conf.with_section(Some("default"))
            .set("access_token", access_auth.access_token)
            .set("refresh_token", access_auth.refresh_token);
        conf.write_to_file(CREDENTIALS_FILE.deref()).unwrap();

        println!("Successfully authenticated!");
    }

    std::process::exit(exitcode::OK)
}
