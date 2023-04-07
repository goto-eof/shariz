use crate::structures::command_processor::CommandProcessor;
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
        println!("server yahooo: {}", &fname);
        let full_path = format!("{}/{}", self.search_directory, &fname);
        println!("full_path: {}", full_path);
        // let mut file = File::open(full_path).unwrap();
        // let mut reader = BufReader::new(file.try_clone().unwrap());
        let mut data = fs::read(full_path).unwrap();

        // write length
        println!("server writing length: {}", data.len());
        stream
            .write_all(format!("{}\r\n", data.len()).as_bytes())
            .unwrap();
        // read OK
        let mut buffer = [0; 100];
        stream.read(&mut buffer);
        let com = str::from_utf8(&buffer).unwrap();
        println!("server ok: {:?}", com);
        if com.starts_with("OK") {
            // write data
            println!("server sending data:  - {}", data.len());
            stream.write_all(&data);
            println!("server data sent");
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
