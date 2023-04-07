use crate::service::file_service::calculate_file_hash;
use crate::structures::config::Config;
use chrono::{Datelike, Timelike, Utc};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::time;
use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use tokio::task::JoinHandle;

pub async fn run_client(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) -> JoinHandle<()> {
    let address = format!("{}:{}", config.target_ip, config.target_port);
    // let shared_directory = format!("{}/client", config.shared_directory);
    let shared_directory = config.shared_directory.clone();
    tokio::spawn(async move {
        let connection = TcpStream::connect(address);

        if connection.is_ok() {
            let mut stream = connection.unwrap();

            let msg = format!("ll \r\n");
            if stream.write_all(msg.as_bytes()).is_ok() {}

            let mut response = String::new();
            let mut conn = BufReader::new(&stream);
            conn.read_line(&mut response);
            let result = response;
            let file_list = result.split(",");
            let count = file_list.clone().count();
            println!("received vector: {:?} - count: {}", file_list, count);

            for file in file_list {
                if file.trim().len() > 0 {
                    println!("\r\n**************************\r\npulling: {}", file);
                    let command = format!("pull {}\r\n", file);
                    println!("command: {}", command);
                    stream.write_all(command.as_bytes()).unwrap();

                    let mut reader = BufReader::new(&stream);
                    let mut buffer = String::new();
                    reader.read_line(&mut buffer);
                    println!("clinet size file: {}", buffer);
                    let mut buffer: Vec<&str> = buffer.split(";").collect();
                    println!("yabadabaduuuu {:?}", buffer);
                    let file_size: u64 = buffer.get(0).unwrap().trim().parse().unwrap();
                    println!("received size: {:?}", buffer);
                    let file_hash = buffer.get(1).unwrap().trim();

                    println!(
                        "========>file_size: {}, file_hash: {}",
                        file_size, file_hash
                    );
                    let fname = Path::new(&file).file_name().unwrap().to_string_lossy();
                    let file_to_save = format!("{}/{}", shared_directory, fname);

                    if !Path::new(&file_to_save).exists()
                        || Path::new(&file_to_save).exists()
                            && file_size != fs::metadata(&file_to_save).unwrap().len()
                        || calculate_file_hash(&file_to_save) != file_hash
                    {
                        stream.write("OK\r\n".as_bytes());
                        println!("client: wating for server file stream....");
                        let mut buffer: Vec<u8> = vec![0; file_size.try_into().unwrap()];
                        stream.read_exact(&mut buffer);
                        println!(
                            "buffer: {:?}, capacity: {}",
                            buffer.capacity(),
                            buffer.capacity()
                        );
                        println!("client red strem file from server");

                        let now = Utc::now();
                        let mut file =
                            File::create(format!("{}/{}", shared_directory, fname)).unwrap();
                        println!("writing on file...");
                        file.write_all(&buffer).unwrap();
                        println!("writed on file");
                        let ten_millis = time::Duration::from_millis(1000);
                    }
                }
            }
            println!("Finished....");
            stream.shutdown(Shutdown::Both);
            return ();
        } else {
            println!("connection error");
            return ();
        }
    })
}
