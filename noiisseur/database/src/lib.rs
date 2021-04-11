mod models;
mod schema;

use std::env;

#[macro_use]
extern crate diesel;

use diesel::{prelude::*, result::QueryResult, sqlite::SqliteConnection};
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
) -> QueryResult<usize> {
    let track = NewTrack {
        spotify_id,
        playlist_id,
        name,
        url,
    };

    diesel::insert_into(tracks::table)
        .values(&track)
        .execute(conn)
}

pub fn delete_track(conn: &SqliteConnection, id: i32) {
    diesel::delete(tracks::table.find(id))
        .execute(conn)
        .expect("Error deleting tracks");
}

pub fn mark_track_as_posted(conn: &SqliteConnection, track: &Track) {
    use crate::schema::tracks::columns::posted;
    diesel::update(track)
        .set(posted.eq(1))
        .execute(conn)
        .expect("Error updating track");
}

pub fn get_tracks(conn: &SqliteConnection) -> Vec<Track> {
    use crate::schema::tracks::columns::posted;
    tracks::table.filter(posted.eq(0)).load::<Track>(conn).unwrap()
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

pub fn insert_playlist_offset(conn: &SqliteConnection, playlist_id: i32, offset: i32) {
    let offset = NewPlaylistOffset {
        offset,
        playlist_id,
    };

    diesel::insert_into(playlist_offset::table)
        .values(&offset)
        .execute(conn)
        .expect("Error inserting playlist offset into database");
}

pub fn update_playlist_offset(conn: &SqliteConnection, playlist_id: i32, offset_val: i32) {
    use crate::schema::playlist_offset::columns::{id, offset};
    diesel::update(playlist_offset::table.filter(id.eq(playlist_id)))
        .set(offset.eq(offset_val))
        .execute(conn)
        .unwrap();
}

pub fn get_playlist_offset(conn: &SqliteConnection, playlist_id_val: i32) -> i32 {
    use crate::schema::playlist_offset::columns::{offset, playlist_id};
    let offset_val = playlist_offset::table
        .filter(playlist_id.eq(playlist_id_val))
        .select(offset)
        .first::<i32>(conn)
        .unwrap();

    offset_val
}
