use std::{fs, io::Write, net::TcpStream};

use crate::{
    service::file_service::extract_fname, structures::command_processor::CommandProcessor,
};

pub struct LLProcessor {
    pub search_directory: String,
}

impl CommandProcessor for LLProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("ll");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        println!("processing command: {}", full_command);
        let mut files_string = "".to_owned();
        let mut lenght = 0;

        let files_result = fs::read_dir(&self.search_directory);
        if files_result.is_err() {
            return false;
        }
        let files = files_result.unwrap();
        for file_result in files {
            if file_result.is_err() {
                return false;
            }
            let file = file_result.unwrap();
            if !file.path().ends_with(".DS_Store") && !file.path().is_dir() {
                files_string = format!(
                    "{}{},",
                    files_string,
                    extract_fname(&file.path().to_string_lossy().to_string())
                );
                lenght = lenght + 1;
            }
        }
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
    pub fn new(directory: &str) -> Self {
        LLProcessor {
            search_directory: directory.to_owned(),
        }
    }
}
