use crate::structures::config::Config;
use chrono::{Datelike, Timelike, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time;
use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use tokio::task::JoinHandle;

pub async fn run_client(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) -> JoinHandle<()> {
    let address = format!("{}:{}", config.target_ip, config.target_port);
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
                    ///////////////////////////////
                    /// pull filename.png
                    ///////////////////////////////
                    println!("\r\n**************************\r\npulling: {}", file);
                    let command = format!("pull {}\r\n", file);
                    println!("command: {}", command);
                    stream.write_all(command.as_bytes()).unwrap();

                    ///////////////////////////////
                    /// read bytes length
                    ///////////////////////////////
                    let mut buffer = [0; 10];
                    let chars = stream.read(&mut buffer);
                    let result = String::from_utf8_lossy(&buffer[0..chars.unwrap()]);
                    println!("parsed size: [{}]", result.to_string().trim());
                    let result: u64 = result.trim().parse().unwrap();

                    println!("received size: {:?}", buffer);
                    stream.write("OK\r\n".as_bytes());

                    println!("client: wating for server file stream....");
                    let mut buffer: Vec<u8> = vec![0; result.try_into().unwrap()];
                    stream.read_exact(&mut buffer);
                    println!(
                        "buffer: {:?}, capacity: {}",
                        buffer.capacity(),
                        buffer.capacity()
                    );
                    println!("client red strem file from server");
                    let fname = Path::new(&file).file_name().unwrap().to_string_lossy();
                    let now = Utc::now();
                    let mut file = File::create(format!(
                        "/Users/andrei/Desktop/shariz/{}-{}-{}_{}-{}-{}_{}-{}",
                        now.year(),
                        now.month(),
                        now.day(),
                        now.hour(),
                        now.minute(),
                        now.second(),
                        now.nanosecond(),
                        fname
                    ))
                    .unwrap();
                    println!("writing on file...");
                    file.write_all(&buffer).unwrap();
                    println!("writed on file");
                    let ten_millis = time::Duration::from_millis(1000);
                }
            }
        }
    })
}
