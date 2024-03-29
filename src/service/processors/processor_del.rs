use diesel::SqliteConnection;
use shariz::models::FileDB;

use crate::{
    dao::file_dao::{delete_file_db, find_file_on_db, DELETED},
    structures::command_processor::CommandProcessor,
};
use std::{
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

        let file_on_db: Option<FileDB> =
            find_file_on_db(&mut self.db_connection_mutex.lock().unwrap(), filename);

        if file_on_db.is_none() {
            println!("server: file not found on db");
            send_response(stream, "KO");
            return false;
        }
        let file_on_db = file_on_db.unwrap();
        if file_on_db.status == DELETED {
            if delete_file_db(
                &mut self.db_connection_mutex.lock().unwrap(),
                &file_on_db.name,
            ) {
                println!("server: record and file deleted successfully");
                send_response(stream, "OK");
                return true;
            } else {
                println!("server: ERROR file not deleted");
                send_response(stream, "KO");
                return false;
            }
        } else {
            println!("server: ERROR file not deleted on db");
            send_response(stream, "KO");
            return false;
        }
    }
}

fn send_response(stream: &mut TcpStream, message: &str) -> bool {
    let write_result = stream.write_all(format!("{}\r\n", message).as_bytes());
    if write_result.is_err() {
        println!(
            "server: error responding to client: {:?}",
            write_result.err()
        );
        return false;
    }
    return true;
}

impl DelProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<SqliteConnection>>) -> Self {
        DelProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
