use diesel::SqliteConnection;

use crate::structures::command_processor::CommandProcessor;
use std::{
    fs,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct DelProcessor {
    pub search_directory: String,
    pub db_connection_mutex: Arc<Mutex<SqliteConnection>>,
}

impl CommandProcessor for DelProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("del");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("server: processing command: {}", full_command);
        let command = full_command.split(";").collect::<Vec<&str>>();
        if command.len() != 2 {
            println!("server: invalid command: {}", full_command);
            return false;
        }
        let filename = command.get(1);
        if filename.is_none() {
            println!("server: invalid command 2: {}", full_command);
        }
        let filename = filename.unwrap();
        let fname = format!("{}/{}", self.search_directory, filename);
        let file_remove_result = fs::remove_file(fname);
        if file_remove_result.is_err() {
            println!(
                "server: error removing file: {:?}",
                file_remove_result.err()
            );
            return false;
        }
        let write_result = stream.write_all("OK\r\n".as_bytes());
        if write_result.is_err() {
            println!(
                "server: error responding to client: {:?}",
                write_result.err()
            );
            return false;
        }

        return true;
    }
}

impl DelProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<SqliteConnection>>) -> Self {
        DelProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
