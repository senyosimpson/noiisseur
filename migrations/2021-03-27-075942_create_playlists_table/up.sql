-- Your SQL goes here
CREATE TABLE playlists (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    spotify_id TEXT NOT NULL UNIQUE
)