-- Your SQL goes here
CREATE TABLE playlist_offsets (
    id INTEGER PRIMARY KEY NOT NULL,
    offset INTEGER NOT NULL,
    playlist_id INTEGER NOT NULL,
    FOREIGN KEY (playlist_id)
        REFERENCES playlists (id)
            ON UPDATE NO ACTION
            ON DELETE CASCADE
)