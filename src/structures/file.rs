use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DbFile {
    pub id: i32,
    pub name: String,
    pub status: i32,
    pub last_update: DateTime<Utc>,
}
