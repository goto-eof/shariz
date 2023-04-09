use crate::{
    service::{
        db_service::{insert_file, list_all_files, update_file_delete_status},
        file_service::extract_fname,
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
        println!("automatic command execution");
        return true;
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("processing command: SYNCHRONIZE");
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
        let files_on_db = list_all_files(&connection).unwrap();
        let files_name_on_db: Vec<String> =
            files_on_db.iter().map(|file| file.name.clone()).collect();

        /* Update deleted files */
        files_on_db.iter().for_each(|file_on_db| {
            if !files_on_disk.contains(&file_on_db.name) {
                if file_on_db.status != 1 {
                    println!("----> delete {}", &file_on_db.name);
                    update_file_delete_status(&connection, (&file_on_db.name).to_string(), 1);
                }
            } else {
                if file_on_db.status != 0 {
                    println!("----> undelete {}", &file_on_db.name);
                    update_file_delete_status(&connection, (&file_on_db.name).to_string(), 0);
                }
            }
        });

        /* add new files */
        for file_on_disk in files_on_disk {
            if !files_name_on_db.contains(&file_on_disk)
                && !Path::new(&format!("{}/{}", self.search_directory, file_on_disk)).is_dir()
            {
                insert_file(&connection, &file_on_disk, 0);
            }
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
