use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::files;

#[derive(Queryable, PartialEq, Debug, Identifiable, AsChangeset)]
#[table_name = "files"]
pub struct FileDB {
    pub id: i32,
    pub name: String,
    pub status: i32,
    pub sha2: String,
    pub last_update: Option<NaiveDateTime>,
}

#[derive(AsChangeset)]
#[table_name = "files"]
pub struct UpdateFileDB {
    pub status: Option<i32>,
    pub sha2: Option<String>,
    pub last_update: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFileDB<'a> {
    pub name: &'a str,
    pub status: i32,
    pub sha2: &'a str,
    pub last_update: Option<&'a NaiveDateTime>,
}
