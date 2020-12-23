mod models;
mod schema;

use std::env;

#[macro_use]
extern crate diesel;

use diesel::{
    prelude::*,
    sqlite::SqliteConnection
};
use dotenv::dotenv;

use models::NewTrack;
use schema::tracks;


pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url))
}

pub fn insert_track<'a>(conn: &SqliteConnection, name: &'a str, url: &'a str) {
    let track = NewTrack {
        name,
        url
    };

    diesel::insert_into(tracks::table)
        .values(&track)
        .execute(conn)
        .expect("Error inserting track into database");
}

pub fn delete_track(conn: &SqliteConnection, id: i32) {
    diesel::delete(tracks::table.find(id))
        .execute(conn)
        .expect("Error deleting posts");
}