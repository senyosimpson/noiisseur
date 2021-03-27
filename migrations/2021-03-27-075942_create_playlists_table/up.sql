-- Your SQL goes here
CREATE TABLE playlists (
    id INTEGER PRIMARY KEY NOT NULL,
    spotify_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
)