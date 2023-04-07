use crate::{
    service::file_service::calculate_file_hash, structures::command_processor::CommandProcessor,
};
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    net::TcpStream,
};
use std::{str, thread, time};

pub struct PullProcessor {
    pub search_directory: String,
}

impl CommandProcessor for PullProcessor {
    fn accept(&self, root_command: &str) -> bool {
        return root_command.starts_with("pull ");
    }

    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool {
        let fname = full_command.split(" ").nth(1).unwrap().trim();
        let full_path = format!("{}/{}", self.search_directory, &fname);
        let sha2 = calculate_file_hash(&full_path);
        // let mut file = File::open(full_path).unwrap();
        // let mut reader = BufReader::new(file.try_clone().unwrap());
        let mut data = fs::read(full_path).unwrap();

        // write length
        stream
            .write_all(format!("{};{}\r\n", data.len(), sha2).as_bytes())
            .unwrap();
        // read OK
        let mut buffer = [0; 100];
        stream.read(&mut buffer);
        let com = str::from_utf8(&buffer).unwrap();
        if com.starts_with("OK") {
            // write data
            stream.write_all(&data);
        } else {
            stream.write("ERROR\r\n".as_bytes());
        }
        return true;
    }
}

impl PullProcessor {
    fn new(directory: &str) -> Self {
        PullProcessor {
            search_directory: directory.to_owned(),
        }
    }
}
