table! {
    playlist_offset (id) {
        id -> Integer,
        playlist_id -> Integer,
        offset -> Integer,
    }
}

table! {
    playlists (id) {
        id -> Integer,
        spotify_id -> Text,
        name -> Text,
    }
}

table! {
    tracks (id) {
        id -> Integer,
        spotify_id -> Text,
        playlist_id -> Integer,
        name -> Text,
        url -> Text,
        posted -> Integer,
    }
}

joinable!(playlist_offset -> playlists (playlist_id));
joinable!(tracks -> playlists (playlist_id));

allow_tables_to_appear_in_same_query!(
    playlist_offset,
    playlists,
    tracks,
);
