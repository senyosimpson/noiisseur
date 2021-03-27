mod models;
mod schema;

use std::{env, iter::Peekable};

#[macro_use]
extern crate diesel;

use diesel::{prelude::*, sqlite::SqliteConnection};
use dotenv::dotenv;

use models::{NewPlaylist, NewPlaylistOffset, NewTrack, Playlist, Track};

use schema::{playlist_offset, playlists, tracks};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url))
}

pub fn insert_track<'a>(
    conn: &SqliteConnection,
    spotify_id: &'a str,
    playlist_id: i32,
    name: &'a str,
    url: &'a str,
) {
    let track = NewTrack {
        spotify_id,
        playlist_id,
        name,
        url,
    };

    diesel::insert_into(tracks::table)
        .values(&track)
        .execute(conn)
        .expect("Error inserting track into database");
}

pub fn delete_track(conn: &SqliteConnection, id: i32) {
    diesel::delete(tracks::table.find(id))
        .execute(conn)
        .expect("Error deleting songs");
}

pub fn get_tracks(conn: &SqliteConnection) -> Vec<Track> {
    tracks::table.load::<Track>(conn).unwrap()
}

pub fn insert_playlist<'a>(conn: &SqliteConnection, name: &'a str, spotify_id: &'a str) -> i32 {
    use crate::schema::playlists::columns::id;

    let playlist = NewPlaylist { name, spotify_id };

    diesel::insert_into(playlists::table)
        .values(&playlist)
        .execute(conn)
        .expect("Error inserting playlist into database");

    let playlist_id = playlists::table
        .select(id)
        .order(id.desc())
        .limit(1)
        .load::<i32>(conn)
        .unwrap()[0];
    
    playlist_id
}

pub fn get_playlists(conn: &SqliteConnection) -> Vec<Playlist> {
    playlists::table.load::<Playlist>(conn).unwrap()
}

pub fn insert_playlist_offset(conn: &SqliteConnection, offset: i32, playlist_id: i32) {
    let offset = NewPlaylistOffset {
        offset,
        playlist_id,
    };

    diesel::insert_into(playlist_offset::table)
        .values(&offset)
        .execute(conn)
        .expect("Error inserting playlist offset into database");
}
