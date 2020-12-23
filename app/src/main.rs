use database::{insert_track, establish_connection};

fn main() {
    let conn = establish_connection();
    let name = String::from("Desires");
    let url = String::from("https://open.spotify.com/track/7eYAHC0RbBF9eaqWzT34Aq?si=upkK5uD8QW6eng3e8FkPEg");

    insert_track(&conn, &name, &url);
}
