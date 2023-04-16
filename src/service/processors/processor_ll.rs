use crate::{dao::file_dao::list_all_files_on_db, structures::command_processor::CommandProcessor};
use diesel::SqliteConnection;
use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct LLProcessor {
    pub search_directory: String,
    db_connection_mutex: Arc<Mutex<SqliteConnection>>,
}

impl CommandProcessor for LLProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("ll");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("server: processing command: {}", full_command);
        let mut files_string = "".to_owned();

        let all_db_files =
            list_all_files_on_db(&mut self.db_connection_mutex.lock().unwrap(), false);
        println!("server: DB result of ll: {}", all_db_files.len());
        all_db_files.iter().for_each(|file| {
            files_string = format!(
                "{}{};{};{};{},",
                files_string,
                file.name,
                file.status,
                file.last_update.unwrap().timestamp_millis(),
                file.sha2
            );
        });

        let files = format!("{}\r\n", files_string);
        let write_result = stream.write_all(files.as_bytes());
        println!("server: writed ll result on buffer");
        if write_result.is_err() {
            return false;
        }
        return true;
    }
}

impl LLProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<SqliteConnection>>) -> Self {
        LLProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
