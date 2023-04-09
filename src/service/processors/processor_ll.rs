use std::{
    fs,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::{
    service::{db_service::list_all_files, file_service::extract_fname},
    structures::{command_processor::CommandProcessor, file::DbFile},
};

pub struct LLProcessor {
    pub search_directory: String,
    db_connection_mutex: Arc<Mutex<Connection>>,
}

impl CommandProcessor for LLProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("ll");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("processing command: {}", full_command);
        let mut files_string = "".to_owned();

        let all_db_files = list_all_files(&self.db_connection_mutex.lock().unwrap()).unwrap();

        all_db_files.iter().for_each(|file| {
            files_string = format!(
                "{}{};{};{},",
                files_string,
                file.name,
                file.status,
                file.last_update.to_rfc3339()
            );
        });

        println!("{}", files_string);
        let files = format!("{}\r\n", files_string);
        let write_result = stream.write_all(files.as_bytes());
        if write_result.is_err() {
            return false;
        }
        return true;
    }
}

impl LLProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<Connection>>) -> Self {
        LLProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
