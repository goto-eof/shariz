use chrono::Utc;
use diesel::{
    insert_into, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection,
};

use shariz::models::{FileDB, NewFileDB, UpdateFileDB};
use shariz::schema::files::dsl::files;
use shariz::schema::files::{name, sha2};

pub const DELETED: i32 = 1;
pub const CREATED: i32 = 0;

pub fn list_all_files_on_db(connection: &mut SqliteConnection) -> Vec<FileDB> {
    let results = files
        .load::<FileDB>(connection)
        .expect("DB: Error loading files");
    return results;
}

pub fn find_file_on_db(connection: &mut SqliteConnection, fname: &str) -> Option<FileDB> {
    let result = files
        .filter(name.eq(fname))
        .first::<FileDB>(connection)
        .optional();
    if result.is_err() {
        return None;
    }
    return result.unwrap();
}

pub fn delete_file_db(connection: &mut SqliteConnection, fname: &str) -> bool {
    let result = files
        .filter(name.eq(fname))
        .first::<FileDB>(connection)
        .optional();
    if result.is_err() {
        return false;
    }
    let result = result.unwrap();
    if result.is_none() {
        println!("DB: file does not exists");
        return false;
    }
    let result = diesel::delete(&result.unwrap()).execute(connection);
    println!("DB: deletion result: {}", result.is_ok());
    return result.is_ok();
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

pub fn update_file_delete_status(
    connection: &mut SqliteConnection,
    fname: String,
    fstatus: i32,
) -> bool {
    let model_db = UpdateFileDB {
        last_update: Some(Utc::now().naive_utc()),
        sha2: None,
        status: Some(fstatus),
    };
    let update_result = diesel::update(files)
        .filter(name.eq(&fname))
        .set(model_db)
        .execute(connection);

    return update_result.is_ok();
}

// pub fn update_file_hash(connection: &mut SqliteConnection, fname: String, fsha2: String) -> bool {
//     let model_db = UpdateFileDB {
//         last_update: None,
//         sha2: Some(fsha2),
//         status: None,
//         frozen: None,
//     };
//     let update_result = diesel::update(files)
//         .filter(name.eq(&fname))
//         .set(model_db)
//         .execute(connection);

//     return update_result.is_ok();
// }

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
    println!("DB: inserted in db: {:?}=>{:?}", fname, result);
    return result.is_ok();
}

// pub fn freeze(connection: &mut SqliteConnection, fname: String, ffreeze: i32) -> bool {
//     let model_db = UpdateFileDB {
//         last_update: Some(Utc::now().naive_utc()),
//         sha2: None,
//         frozen: Some(ffreeze),
//         status: None,
//     };
//     let update_result = diesel::update(files)
//         .filter(name.eq(&fname))
//         .set(model_db)
//         .execute(connection);

//     return update_result.is_ok();
// }
