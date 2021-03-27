table! {
    playlist_offsets (id) {
        id -> Integer,
        offset -> Integer,
        playlist_id -> Integer,
    }
}

table! {
    playlists (id) {
        id -> Integer,
        name -> Text,
        spotify_id -> Text,
    }
}

table! {
    tracks (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
    }
}

joinable!(playlist_offsets -> playlists (playlist_id));

allow_tables_to_appear_in_same_query!(
    playlist_offsets,
    playlists,
    tracks,
);
