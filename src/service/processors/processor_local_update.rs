use crate::{
    service::{
        db_service::{self, insert_file, list_all_files_on_db, update_file_delete_status},
        file_service::{calculate_file_hash, extract_fname},
    },
    structures::command_processor::CommandProcessor,
};
use rusqlite::Connection;
use std::{
    fs,
    net::TcpStream,
    path::Path,
    sync::{Arc, Mutex},
};

pub struct LocalUpdateProcessor {
    pub search_directory: String,
    db_connection_mutex: Arc<Mutex<Connection>>,
}

impl CommandProcessor for LocalUpdateProcessor {
    fn accept(&self, root_command: &str) -> bool {
        println!("automatic command execution: {:?}", root_command);
        return true;
    }

    fn process(&self, _full_command: &str, _stream: &mut TcpStream) -> bool {
        let search_directory = self.search_directory.clone();
        let connection = self.db_connection_mutex.lock();
        if connection.is_err() {
            println!("error db connection");
            return false;
        }
        let connection = connection.unwrap();
        return LocalUpdateProcessor::sync_disk_with_db(&connection, search_directory.as_str());
    }
}

impl LocalUpdateProcessor {
    pub fn new(directory: &str, db_connection_mutex: Arc<Mutex<Connection>>) -> Self {
        LocalUpdateProcessor {
            search_directory: directory.to_owned(),
            db_connection_mutex,
        }
    }

    pub fn sync_disk_with_db(connection: &Connection, search_directory: &str) -> bool {
        println!("processing command: SYNCHRONIZE");
        let files_result = fs::read_dir(&search_directory);
        if files_result.is_err() {
            return false;
        }
        let files = files_result.unwrap();
        let mut files_on_disk: Vec<String> = vec![];
        for file_result in files {
            if file_result.is_err() {
                println!("unable to list file");
                return false;
            }
            let file = file_result.unwrap();
            let file_name = extract_fname(&file.path().to_string_lossy().to_string());
            files_on_disk.push(file_name);
        }
        let files_on_db = list_all_files_on_db(&connection).unwrap();
        let files_name_on_db: Vec<String> =
            files_on_db.iter().map(|file| file.name.clone()).collect();
        for file_on_disk in files_on_disk.clone() {
            let full_path = format!("{}/{}", search_directory, file_on_disk);
            if !files_name_on_db.contains(&file_on_disk)
                && !Path::new(&full_path).is_dir()
                && !file_on_disk.eq(".DS_Store")
            {
                let sha2 = calculate_file_hash(&full_path);
                if sha2.is_none() {
                    println!("unable to calculate sha2 of file: {}", file_on_disk);
                } else {
                    insert_file(&connection, &file_on_disk, 0, sha2.unwrap().as_str());
                }
            }
        }
        files_on_db.iter().for_each(|file_on_db| {
            if !files_on_disk.contains(&file_on_db.name) {
                if file_on_db.status != db_service::DELETED {
                    println!("----> delete {}", &file_on_db.name);
                    update_file_delete_status(
                        &connection,
                        (&file_on_db.name).to_string(),
                        db_service::DELETED,
                    );
                }
            } else {
                if file_on_db.status != db_service::CREATED {
                    println!("----> undelete {}", &file_on_db.name);
                    update_file_delete_status(
                        &connection,
                        (&file_on_db.name).to_string(),
                        db_service::CREATED,
                    );
                }
            }
        });

        return true;
    }
}
