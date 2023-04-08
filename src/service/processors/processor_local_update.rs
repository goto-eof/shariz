use std::{
    fs,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::{
    service::file_service::extract_fname, structures::command_processor::CommandProcessor,
};

pub struct LocalUpdateProcessor {
    pub search_directory: String,
    db_connection_mutex: Arc<Mutex<Connection>>,
}

impl CommandProcessor for LocalUpdateProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return true;
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("processing command: {}", full_command);
        let files_result = fs::read_dir(&self.search_directory);
        if files_result.is_err() {
            return false;
        }
        let connection = self.db_connection_mutex.lock();
        if connection.is_err() {
            println!("unable to get the db connection");
            return false;
        }
        let connection = connection.unwrap();
        let files = files_result.unwrap();
        for file_result in files {
            if file_result.is_err() {
                println!("unable to list files");
                return false;
            }
            let file = file_result.unwrap();
            let file_name = extract_fname(&file.path().to_string_lossy().to_string());
        }

        return true;
    }
}

impl LocalUpdateProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<Connection>>) -> Self {
        LocalUpdateProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
