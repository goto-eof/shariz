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
        // let mut lenght = 0;

        // let files_result = fs::read_dir(&self.search_directory);
        // if files_result.is_err() {
        //     return false;
        // }
        // let files = files_result.unwrap();
        let all_db_files = list_all_files(&self.db_connection_mutex.lock().unwrap()).unwrap();

        all_db_files.iter().for_each(|file| {
            files_string = format!("{}{};{},", files_string, file.name, file.status);
        });
        // for file_result in files {
        //     if file_result.is_err() {_
        //         return false;
        //     }
        //     let file = file_result.unwrap();

        //     if !file.path().ends_with(".DS_Store") && !file.path().is_dir() {
        //         let file_name = extract_fname(&file.path().to_string_lossy().to_string());
        //         let status_vec: Vec<&DbFile> = all_db_files
        //             .iter()
        //             .filter(|file| file.name.eq(&file_name))
        //             .collect();
        //         let mut status = 0;
        //         if status_vec.len() > 0 {
        //             status = status_vec.get(0).unwrap().status;
        //         }
        //         files_string = format!("{}{};{},", files_string, file_name, status);
        //         lenght = lenght + 1;
        //     }
        // }
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
