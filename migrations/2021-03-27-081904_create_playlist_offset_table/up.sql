-- Your SQL goes here
CREATE TABLE playlist_offset (
    id INTEGER PRIMARY KEY NOT NULL,
    playlist_id INTEGER NOT NULL,
    offset INTEGER NOT NULL,
    FOREIGN KEY (playlist_id)
        REFERENCES playlists (id)
            ON UPDATE NO ACTION
            ON DELETE CASCADE
)