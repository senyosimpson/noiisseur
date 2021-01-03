use diesel::{Insertable, Queryable};
use crate::schema::tracks;


#[derive(Queryable, PartialEq)]
pub struct Track {
    pub id: i32,
    pub name: String,
    pub url: String
}

#[derive(Insertable)]
#[table_name="tracks"]
pub struct NewTrack<'a> {
    pub name: &'a str,
    pub url: &'a str
}