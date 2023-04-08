use rusqlite::Connection;

use crate::{
    service::file_service::calculate_file_hash, structures::command_processor::CommandProcessor,
};
use std::str;
use std::sync::{Arc, Mutex};
use std::{
    fs::{self},
    io::{Read, Write},
    net::TcpStream,
};

pub struct PullProcessor {
    pub search_directory: String,
    pub db_connection_mutex: Arc<Mutex<Connection>>,
}

impl CommandProcessor for PullProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("pull");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("processing command: {}", full_command);
        if full_command.len() == 0 {
            println!("error: command_length is 0");
            return false;
        }
        let fname = full_command.split(";").collect::<Vec<&str>>();
        if fname.len() != 2 {
            println!("error: invalid command");
            return false;
        }

        let fname = fname.get(1).unwrap().trim();

        let full_path = format!("{}/{}", self.search_directory, &fname);
        let sha2 = calculate_file_hash(&full_path);
        let data = fs::read(full_path).unwrap();

        stream
            .write_all(format!("{};{}\r\n", data.len(), sha2.unwrap()).as_bytes())
            .unwrap();
        let mut buffer = [0; 100];
        let read_result = stream.read(&mut buffer);

        if read_result.is_err() {
            println!("error: client did not send OK");
            return false;
        }

        let from_utf8_result = str::from_utf8(&buffer);
        if from_utf8_result.is_err() {
            println!(
                "error: client did not send OK (2): {:?}",
                from_utf8_result.err()
            );
            return false;
        }
        let client_response = from_utf8_result.unwrap();

        if client_response.starts_with("OK") {
            // write data
            let write_result = stream.write_all(&data);
            if write_result.is_err() {
                println!("error: client did not send OK (4)");
                return false;
            }
        } else {
            println!("error: client did not send OK (3)");
        }
        return true;
    }
}

impl PullProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<Connection>>) -> Self {
        PullProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }
}
