use anyhow::{Context, Result};
use diesel::result::{DatabaseErrorKind, Error};
use dotenv::dotenv;
use oauth;
use rand::Rng;
use reqwest::{blocking::Client, header};
use std::{collections::HashMap, env};
use structopt::StructOpt;

use database::{
    self, establish_connection, get_playlist_offset, get_playlists, insert_playlist,
    insert_playlist_offset, insert_track, mark_track_as_posted, update_playlist_offset,
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
    Tracks(TrackCmd),
    /// All commands related to playlists
    Playlist(PlaylistCmd),
}

#[derive(Debug, StructOpt)]
enum TrackCmd {
    // Posts the song to Twitter
    Post,
    // Updates the songs in the database
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
    spotify_id: String,
}

fn main() -> Result<()> {
    dotenv().ok();

    let conn = establish_connection().with_context(|| "Could not establish connection!")?;
    match Command::from_args() {
        Command::Auth => {
            authenticate()?;
            Ok(())
        }
        Command::Playlist(playlist_cmd) => match playlist_cmd {
            PlaylistCmd::Add(PlaylistInfo { name, spotify_id }) => {
                let playlist_id = insert_playlist(&conn, &name, &spotify_id);
                insert_playlist_offset(&conn, playlist_id, 0);
                println!("Added playlist {} with id {}", name, spotify_id);
                Ok(())
            }
            PlaylistCmd::Remove => Ok(()),
        },
        Command::Tracks(track_cmd) => match track_cmd {
            TrackCmd::Post => {
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

                let conn =
                    establish_connection().with_context(|| "Could not establish connection!")?;
                let tracks = database::get_tracks(&conn);
                let idx: usize = rand::thread_rng().gen_range(0..tracks.len());
                let track = tracks.get(idx).unwrap(); // This should never fail so can unwrap

                let client = Client::new();
                let request = Tweet {
                    status: track.url.clone(),
                };
                // Creates the authentication header
                let auth_header = oauth::post(POST_TWEET_URL, &request, &token, oauth::HmacSha1);

                // Tweet the song
                let mut params = HashMap::new();
                params.insert("status", track.url.clone());
                let response = client
                    .post(POST_TWEET_URL)
                    .header(header::AUTHORIZATION, auth_header)
                    .form(&params)
                    .send()?;

                if response.status().is_success() {
                    mark_track_as_posted(&conn, track);
                    println!("Successfully tweeted song: {}", track.name);
                    Ok(())
                } else {
                    println!("Failed to tweet song: {}", track.name);
                    println!("Got response: {}", response.text().unwrap());
                    std::process::exit(1)
                }
            }
            TrackCmd::Update => {
                let access_token = refresh_access_token()?;
                let conn =
                    establish_connection().with_context(|| "Could not establish connection!")?;

                let playlists = get_playlists(&conn);
                for (idx, playlist) in playlists.iter().enumerate() {
                    println!(
                        "Processing playlist: {} - [{}]/[{}]",
                        playlist.name,
                        idx + 1,
                        playlists.len()
                    );
                    let offset = get_playlist_offset(&conn, playlist.id);
                    let tracks = spotify::get_tracks(&access_token, &playlist.spotify_id, offset)
                        .with_context(|| "Unable to get fetch all tracks from Spotify")?;

                    for (idx, track) in tracks.iter().enumerate() {
                        println!("Inserting track [{}]/[{}]", idx, tracks.len());
                        if track.is_null() {
                            continue;
                        };
                        let result = insert_track(
                            &conn,
                            &track.spotify_id().unwrap(),
                            playlist.id,
                            &track.name().unwrap(),
                            &track.url().unwrap(),
                        );

                        match result {
                            Ok(_) => continue,
                            // It's possible for the same song to exist in multiple playlists. Currently,
                            // we don't actually want to store duplicates in the `tracks` table. Therefore
                            // we just ignore these cases. They're not common so this solution is fine
                            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                                continue
                            }
                            Err(e) => panic!("Error {} occurred", e),
                        }
                    }

                    let new_offset = offset + tracks.len() as i32;
                    update_playlist_offset(&conn, playlist.id, new_offset);
                }
                Ok(())
            }
        },
    }
}
