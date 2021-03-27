use dotenv::dotenv;
use oauth;
use std::{collections::HashMap, env};
use structopt::StructOpt;

use reqwest::{blocking::Client, header};

use database::{
    self, establish_connection, get_playlists, insert_playlist, insert_playlist_offset,
    insert_track,
};
use spotify::{self, authenticate, refresh_access_token};

const POST_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/update.json";

#[derive(oauth::Request)]
struct Tweet {
    status: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Noiisseur", about = "Options for running Noiisseur.")]
enum Command {
    /// Authenticate application
    Auth,
    /// All commands related to records
    Records(RecordCmd),
    /// All commands related to playlists
    Playlist(PlaylistCmd),
}

#[derive(Debug, StructOpt)]
enum RecordCmd {
    Post,
    Update,
}

#[derive(Debug, StructOpt)]
enum PlaylistCmd {
    Add(PlaylistInfo),
    Remove,
}

#[derive(Debug, StructOpt)]
struct PlaylistInfo {
    name: String,
    spotify_id: String
}

fn main() {
    dotenv().ok();

    let conn = establish_connection();
    match Command::from_args() {
        Command::Auth => authenticate(),
        Command::Playlist(playlist_cmd) => match playlist_cmd {
            PlaylistCmd::Add(PlaylistInfo { name, spotify_id }) => {
                let playlist_id = insert_playlist(&conn, &name, &spotify_id);
                insert_playlist_offset(&conn, 0, playlist_id);
                println!("Added playlist {} with id {}", name, spotify_id)
            }
            PlaylistCmd::Remove => {}
        },
        Command::Records(record_cmd) => match record_cmd {
            RecordCmd::Post => {
                let twitter_consumer_key = env::var("TWITTER_CONSUMER_KEY")
                    .expect("Missing environment variable: TWITTER_CONSUMER_KEY");
                let twitter_consumer_secret = env::var("TWITTER_CONSUMER_SECRET")
                    .expect("Missing environment variable: TWITTER_CONSUMER_SECRET");
                let twitter_access_token = env::var("TWITTER_ACCESS_TOKEN")
                    .expect("Missing environment variable: TWITTER_ACCESS_TOKEN");
                let twitter_access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")
                    .expect("Missing environment variable: TWITTER_ACCESS_TOKEN_SECRET");

                let token = oauth::Token::from_parts(
                    twitter_consumer_key,
                    twitter_consumer_secret,
                    twitter_access_token,
                    twitter_access_token_secret,
                );

                let conn = establish_connection();
                // let ref ?
                let tracks = database::get_tracks(&conn);
                let track = tracks.get(0).unwrap(); // Get rid of this unwrap

                let client = Client::new();
                let request = Tweet {
                    status: track.url.clone(),
                };
                let auth_header = oauth::post(POST_TWEET_URL, &request, &token, oauth::HmacSha1);

                let mut params = HashMap::new();
                params.insert("status", track.url.clone());
                let response = client
                    .post(POST_TWEET_URL)
                    .header(header::AUTHORIZATION, auth_header)
                    .form(&params)
                    .send()
                    .unwrap();

                if response.status().is_success() {
                    println!("Successfully tweeted song: {}", track.name);
                } else {
                    println!("Failed to tweet song: {}", track.name);
                }
            }
            RecordCmd::Update => {
                let access_token = refresh_access_token();
                let conn = establish_connection();
                // Fetch playlists from database
                let playlists = get_playlists(&conn);

                // for playlist in playlists.iter() {
                //     let tracks = spotify::get_tracks(playlist.id, &access_token);
                //     // TODO: Make this an upsert
                //     for track in tracks {
                //         insert_track(&conn, &track.name(), &track.url());
                //     }
                // }
            }
        },
    }
}
