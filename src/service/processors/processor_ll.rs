use std::{fs, io::Write, net::TcpStream};

use crate::{
    service::file_service::extract_fname, structures::command_processor::CommandProcessor,
};

pub struct LLProcessor {
    pub search_directory: String,
}

impl CommandProcessor for LLProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("ll ");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        let mut files = "".to_owned();
        let mut lenght = 0;
        for file in fs::read_dir(&self.search_directory).unwrap() {
            let file = file.unwrap();
            if !file.path().ends_with(".DS_Store") && !file.path().is_dir() {
                files = format!(
                    "{}{},",
                    files,
                    extract_fname(&file.path().to_string_lossy().to_string())
                );
                lenght = lenght + 1;
            }
        }

        let files = format!("{}\r\n", files);
        stream.write_all(files.as_bytes()).unwrap();
        return true;
    }
}

impl LLProcessor {
    fn new(directory: &str) -> Self {
        LLProcessor {
            search_directory: directory.to_owned(),
        }
    }
}
