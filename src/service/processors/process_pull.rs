use crate::{
    service::file_service::calculate_file_hash, structures::command_processor::CommandProcessor,
};
use std::str;
use std::{
    fs::{self},
    io::{Read, Write},
    net::TcpStream,
};

pub struct PullProcessor {
    pub search_directory: String,
}

impl CommandProcessor for PullProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("pull ");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        if full_command.len() == 0 {
            return false;
        }
        let fname = full_command.split(";").collect::<Vec<&str>>();
        if fname.len() != 2 {
            return false;
        }

        let fname = fname.get(1).unwrap().trim();

        let full_path = format!("{}/{}", self.search_directory, &fname);
        let sha2 = calculate_file_hash(&full_path);
        let data = fs::read(full_path).unwrap();

        stream
            .write_all(format!("{};{}\r\n", data.len(), sha2).as_bytes())
            .unwrap();
        let mut buffer = [0; 100];
        let read_result = stream.read(&mut buffer);

        if read_result.is_err() {
            return false;
        }

        let from_utf8_result = str::from_utf8(&buffer);
        if from_utf8_result.is_err() {
            return false;
        }
        let client_response = from_utf8_result.unwrap();

        if client_response.starts_with("OK") {
            // write data
            let write_result = stream.write_all(&data);
            if write_result.is_err() {
                return false;
            }
        }
        return true;
    }
}

impl PullProcessor {
    pub fn new(directory: &str) -> Self {
        PullProcessor {
            search_directory: directory.to_owned(),
        }
    }
}
