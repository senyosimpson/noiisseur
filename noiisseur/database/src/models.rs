use crate::schema::{playlist_offset, playlists, tracks};
use diesel::{Insertable, Queryable};

#[derive(Queryable, Identifiable, PartialEq)]
pub struct Track {
    pub id: i32,
    pub spotify_id: String,
    pub playlist_id: i32,
    pub name: String,
    pub url: String,
    pub posted: i32,
}

#[derive(Insertable)]
#[table_name = "tracks"]
pub struct NewTrack<'a> {
    pub spotify_id: &'a str,
    pub playlist_id: i32,
    pub name: &'a str,
    pub url: &'a str,
}

#[derive(Queryable, PartialEq)]
pub struct Playlist {
    pub id: i32,
    pub spotify_id: String,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "playlists"]
pub struct NewPlaylist<'a> {
    pub name: &'a str,
    pub spotify_id: &'a str,
}

#[derive(Queryable, PartialEq)]
pub struct PlaylistOffset {
    pub id: i32,
    pub offset: i32,
    pub playlist_id: i32,
}

#[derive(Insertable)]
#[table_name = "playlist_offset"]
pub struct NewPlaylistOffset {
    pub offset: i32,
    pub playlist_id: i32,
}
