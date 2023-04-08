use rusqlite::Connection;

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
             name text not null,
             status integer not null
         )",
        [],
    );

    if create_table_statement_result.is_err() {
        println!("unable to initialize db (2)");
        return None;
    }

    return Some(conn);
}

pub fn list_all_files(connection: Connection) -> Option<Vec<(i32, String, i32)>> {
    let mut statement_result = connection.prepare("SELECT f.id, f.name, f.status from files f");
    if statement_result.is_err() {
        println!("unable to extract filenames from db");
        return None;
    }
    let mut statement = statement_result.unwrap();
    let files = statement.query_map([], |row| {
        Ok((
            row.get::<usize, i32>(0).unwrap(),
            row.get::<usize, String>(1).unwrap(),
            row.get::<usize, i32>(2).unwrap(),
        ))
    });
    let files = files.unwrap();
    let files: Vec<(i32, String, i32)> = files.map(|item| item.unwrap()).collect();
    return Some(files);
}
