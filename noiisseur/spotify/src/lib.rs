#![feature(proc_macro_hygiene, decl_macro)]

pub mod error;

use std::{env, path::PathBuf};

use anyhow::Result;
use base64;
use csrf::{CsrfProtection, HmacCsrfProtection};
use dirs::home_dir;
use form_urlencoded;
use hmac::{Hmac, Mac, NewMac};
use ini::Ini;
use lazy_static::{__Deref, lazy_static};
use reqwest::{blocking::Client, header};
use ring::rand::{self, SecureRandom};
use rocket::response::content::Html;
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
const SUCCESS_PAGE: &str = include_str!("../../../html/success.html");
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

#[derive(Debug, Deserialize)]
struct ExternalUrl {
    #[serde(rename = "spotify")]
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyTrackInner {
    #[serde(rename = "id")]
    pub spotify_id: String,
    pub name: String,
    #[serde(rename = "external_urls")]
    url: ExternalUrl,
}
#[derive(Debug, Deserialize)]
pub struct SpotifyTrack {
    #[serde(rename = "track")]
    pub track: Option<SpotifyTrackInner>,
}

impl SpotifyTrack {
    pub fn is_null(&self) -> bool {
        match self.track {
            Some(_) => return false,
            None => return true,
        };
    }

    pub fn spotify_id(&self) -> Option<String> {
        if self.is_null() {
            return None;
        };

        let ref track = self.track.as_ref().unwrap();
        let spotify_id = track.spotify_id.clone();
        Some(spotify_id)
    }

    pub fn name(&self) -> Option<String> {
        if self.is_null() {
            return None;
        };

        let track = self.track.as_ref().unwrap();
        let name = track.name.clone();
        Some(name)
    }

    pub fn url(&self) -> Option<String> {
        if self.is_null() {
            return None;
        };
        // Need to figure out how to restructure my coe
        // to remove this
        let track = self.track.as_ref().unwrap();
        let url = track.url.url.clone();
        Some(url)
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

pub fn refresh_access_token() -> Result<String> {
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
        .send()?
        .text()?;

    let refresh_auth: SpotifyRefreshAuth = serde_json::from_str(&response)?;

    credentials
        .with_section(Some("default"))
        .set("access_token", refresh_auth.access_token);
    credentials.write_to_file(CREDENTIALS_FILE.deref()).unwrap();

    Ok(refresh_auth.access_token.to_string())
}

pub fn get_tracks(access_token: &str, playlist_id: &str, offset: i32) -> Result<Vec<SpotifyTrack>> {
    let mut tracks_url = format!(
        "https://api.spotify.com/v1/playlists/{playlist_id}/\
        tracks?fields=next,items(track(id,name,external_urls))\
        &offset={offset}",
        playlist_id = playlist_id,
        offset = offset
    );

    let client = Client::new();
    let mut tracks: Vec<SpotifyTrack> = Vec::new();

    // Paginate
    loop {
        let mut response = {
            let response = client
                .get(&tracks_url)
                .bearer_auth(access_token)
                .send()?
                .text()?;

            // TODO: Improve error handling
            let response: SpotifyTrackPage = serde_json::from_str(&response)?;
            response
        };
        tracks.append(&mut response.tracks);

        tracks_url = match response.next {
            Some(url) => String::from(url),
            None => break,
        };
    }

    Ok(tracks)
}

pub fn authenticate() -> Result<()> {
    let client_id = env::var("SPOTIFY_CLIENT_ID")?;
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

    webbrowser::open(&url)?;
    let error = rocket::ignite().mount("/", routes![_authenticate]).launch();
    Err(error.into())
}

#[get("/auth?<code>&<state>")]
fn _authenticate(code: String, state: String) -> Result<Html<&'static str>> {
    if state == *STATE {
        let client_id = env::var("SPOTIFY_CLIENT_ID")?;
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET")?;

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
            .send()?
            .text()?;

        let access_auth: SpotifyAccessAuth = serde_json::from_str(&response)?;

        let mut conf = Ini::new();
        conf.with_section(Some("default"))
            .set("access_token", access_auth.access_token)
            .set("refresh_token", access_auth.refresh_token);
        conf.write_to_file(CREDENTIALS_FILE.deref())?;

        println!("Successfully authenticated!");
    } else {
        return Err(error::Error::InvalidOAuthState.into());
    }

    Ok(Html(SUCCESS_PAGE))
}
