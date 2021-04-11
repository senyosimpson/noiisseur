-- Your SQL goes here
CREATE TABLE tracks (
    id INTEGER PRIMARY KEY NOT NULL,
    spotify_id TEXT NOT NULL UNIQUE,
    playlist_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    posted INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (playlist_id)
        REFERENCES playlists (id)
            ON UPDATE NO ACTION
            ON DELETE CASCADE
)