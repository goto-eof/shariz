use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

use shariz::models::{FileDB, NewFileDB, UpdateFileDB};
use shariz::schema::files::dsl::files;
use shariz::schema::files::{last_update, name, sha2, status};
use shariz::schema::{self};

use crate::structures::file;
pub const DELETED: i32 = 1;
pub const CREATED: i32 = 0;

pub fn list_all_files_on_db(connection: &mut SqliteConnection) -> Vec<FileDB> {
    let results = files
        .load::<FileDB>(connection)
        .expect("Error loading posts");
    return results;
}

pub fn retrieve_file_hash_from_db(
    connection: &mut SqliteConnection,
    fname: &str,
) -> Option<String> {
    let sha2_result = files
        .select(sha2)
        .filter(name.eq(fname))
        .first::<String>(connection);
    if sha2_result.is_err() {
        return None;
    }
    return Some(sha2_result.unwrap());
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

pub fn update_file_delete_status(
    connection: &mut SqliteConnection,
    fname: String,
    fstatus: i32,
) -> bool {
    let file_on_db: FileDB = files.filter(name.eq(&fname)).first(connection).unwrap();

    let file = FileDB {
        id: file_on_db.id,
        last_update: Some(Utc::now().naive_utc()),
        status: fstatus,
        name: fname,
        sha2: file_on_db.sha2,
    };

    let post = diesel::update(files).set(file).execute(connection);
    return post.is_ok();
}

pub fn update_file_sha2(connection: &mut SqliteConnection, fname: String, fsha2: String) -> bool {
    let file = files.filter(name.eq(&fname)).first(connection);
    let file: FileDB = file.unwrap();
    let modelDB = UpdateFileDB {
        last_update: Some(Utc::now().naive_utc()),
        sha2: Some(fsha2),
        status: Some(file.status),
    };
    let update_result = diesel::update(files)
        .filter(name.eq(&fname))
        .set(modelDB)
        .execute(connection);

    return update_result.is_ok();
}

pub fn insert_file(
    connection: &mut SqliteConnection,
    fname: &str,
    fstatus: i32,
    fsha2: &str,
) -> bool {
    let result = insert_into(files)
        .values(&NewFileDB {
            last_update: Some(&Utc::now().naive_utc()),
            name: fname,
            sha2: fsha2,
            status: fstatus,
        })
        .execute(connection);

    return result.is_ok();
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
