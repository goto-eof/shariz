use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::structures::file::DbFile;

pub fn initialize_db() -> Option<Connection> {
    let conn = Connection::open("shariz.db");
    if conn.is_err() {
        println!("Unable to initialize db");
        return None;
    }
    let conn = conn.unwrap();
    let create_table_statement_result = conn.execute(
        "create table if not exists files (
             id integer primary key,
             name text not null unique,
             status integer not null,
             sha2 text not null,
             last_update timestamp
         )",
        [],
    );

    if create_table_statement_result.is_err() {
        println!("unable to initialize db (2)");
        return None;
    }

    return Some(conn);
}

pub fn list_all_files(connection: &Connection) -> Option<Vec<DbFile>> {
    let statement_result =
        connection.prepare("SELECT f.id, f.name, f.status, f.last_update, f.sha2 from files f");
    if statement_result.is_err() {
        println!("unable to extract filenames from db");
        return None;
    }
    let mut statement = statement_result.unwrap();
    let files = statement.query_map([], |row| {
        Ok(DbFile {
            id: row.get::<usize, i32>(0).unwrap(),
            name: row.get::<usize, String>(1).unwrap(),
            status: row.get::<usize, i32>(2).unwrap(),
            last_update: row.get::<usize, DateTime<Utc>>(3).unwrap(),
            sha2: row.get::<usize, String>(4).unwrap(),
        })
    });
    let files = files.unwrap();
    let files: Vec<DbFile> = files.map(|item| item.unwrap()).collect();
    return Some(files);
}

pub fn retrieve_file_hash_from_db(connection: &Connection, fname: &str) -> Option<String> {
    let mut stmt = connection.prepare("SELECT sha2 FROM files");
    if stmt.is_err() {
        println!("unable to retrieve sha2 (1)");
        return None;
    }
    let mut stmt = stmt.unwrap();
    let mut rows = stmt.query([]);
    if rows.is_err() {
        println!("unable to retrieve sha2 (1)");
        return None;
    }
    let mut rows = rows.unwrap();
    return Some(rows.next().unwrap().unwrap().get(0).unwrap());
}

// pub fn retrieve_deleted_files(connection: &Connection) -> Option<Vec<String>> {
//     let mut statement_result = connection.prepare("SELECT f.name from files f");
//     if statement_result.is_err() {
//         println!("unable to extract filenames from db");
//         return None;
//     }
//     let mut statement = statement_result.unwrap();
//     let files = statement.query_map([], |row| Ok(row.get::<usize, String>(1).unwrap()));
//     let files = files.unwrap();
//     let files: Vec<String> = files.map(|item| item.unwrap()).collect();
//     return Some(files);
// }

pub fn update_file_delete_status(connection: &Connection, name: String, status: i32) -> bool {
    let statement_result = connection
        .prepare_cached("UPDATE files SET status=?1, last_update=CURRENT_TIMESTAMP WHERE name=?2");
    if statement_result.is_err() {
        println!("unable to udpate file status: {:?}", statement_result.err());
        return false;
    }
    let statement_result = statement_result
        .unwrap()
        .execute(rusqlite::params![status, name]);
    println!("updated file status: {} - {}", name, status);
    return statement_result.unwrap() == 1;
}

pub fn update_file_sha2(connection: &Connection, name: String, sha2: String) -> bool {
    let statement_result = connection
        .prepare_cached("UPDATE files SET sha2=?1, last_update=CURRENT_TIMESTAMP WHERE name=?2");
    if statement_result.is_err() {
        println!("unable to udpate file status: {:?}", statement_result.err());
        return false;
    }
    let statement_result = statement_result
        .unwrap()
        .execute(rusqlite::params![sha2, name]);
    println!("updated file sha2: {} - {}", name, sha2);
    return statement_result.unwrap() == 1;
}

pub fn insert_file(connection: &Connection, fname: &str, status: i32, sha2: &str) -> bool {
    let result = connection.execute(
        "INSERT INTO files (name, status, last_update, sha2) values (?1, ?2, CURRENT_TIMESTAMP, ?3)",
        &[&fname.to_string(), &status.to_string(), &sha2.to_string()],
    );
    if result.is_err() {
        println!("error inserting filename");
        return false;
    }
    return true;
}

// pub fn check_if_exists(connection: &Connection, fname: &str) -> bool {
//     let stmt = connection.prepare("SELECT EXISTS(SELECT 1 FROM files WHERE name=?1)");
//     if stmt.is_err() {
//         println!("error while checking if reord exists");
//         return false;
//     }
//     let mut stmt = stmt.unwrap();
//     let query = stmt.query(rusqlite::params![fname]);
//     if query.is_err() {
//         println!("unable to check existence of record");
//         return false;
//     }
//     let mut query = query.unwrap();
//     return query
//         .next()
//         .unwrap()
//         .unwrap()
//         .get::<usize, bool>(0)
//         .unwrap();
// }
