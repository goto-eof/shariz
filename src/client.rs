use crate::service::file_service::calculate_file_hash;
use crate::structures::config::Config;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;
use tokio::task::JoinHandle;

pub async fn run_client(
    config: &Config,
    db_connection_mutex: Arc<Mutex<Connection>>,
) -> JoinHandle<()> {
    let address = format!("{}:{}", config.target_ip, config.target_port);
    let shared_directory = config.shared_directory.clone();
    let rd_timeout = config.client_rd_timeout;
    let wr_timeout = config.client_wr_timeout;
    tokio::spawn(async move {
        let connection = TcpStream::connect(address);

        if connection.is_ok() {
            let stream = connection.unwrap();
            let result_rt = stream.set_read_timeout(Some(Duration::from_millis(rd_timeout)));
            if result_rt.is_err() {
                let result_shutdown = stream.shutdown(Shutdown::Both);
                if result_shutdown.is_err() {
                    println!("shutdown error");
                }
            }
            let result_wt = stream.set_write_timeout(Some(Duration::from_millis(wr_timeout)));
            if result_wt.is_err() {
                let result_shutdown = stream.shutdown(Shutdown::Both);
                if result_shutdown.is_err() {
                    println!("shutdown error");
                }
            }
            let mut cloned_stream = stream.try_clone().unwrap();

            request_for_file_list(&mut cloned_stream);

            let file_list = read_file_list(&stream);

            for file in file_list {
                if file.trim().len() > 0 {
                    make_pull_request(file.as_str(), &mut cloned_stream);

                    let opt_file_size_hash = size_sha2_request(&stream);
                    if opt_file_size_hash.is_none() {
                        send_ko(&mut cloned_stream);
                    }
                    let (file_size, file_hash) = opt_file_size_hash.unwrap();
                    let (fname, file_to_save) =
                        calculate_file_to_save(file.as_str(), &shared_directory);

                    if !Path::new(&file_to_save).exists()
                        || Path::new(&file_to_save).exists()
                            && file_size != fs::metadata(&file_to_save).unwrap().len()
                        || calculate_file_hash(&file_to_save) != file_hash
                    {
                        send_data_request(&mut cloned_stream);
                        let buffer = extract_file_from_stream(file_size, &mut cloned_stream);
                        override_file(buffer, &shared_directory, fname);
                    } else {
                        send_ko(&mut cloned_stream);
                    }
                }
            }
            let result_shutdown = stream.shutdown(Shutdown::Both);
            if result_shutdown.is_err() {
                println!("shutdown error");
            }
            return ();
        } else {
            println!("connection error");
            return ();
        }
    })
}

fn send_ko(cloned_stream: &mut TcpStream) {
    let write_result = cloned_stream.write("KO\r\n".as_bytes());
    if write_result.is_err() {
        println!("can't send respond to server: {:?}", write_result);
    }
}

fn read_file_list(stream: &TcpStream) -> Vec<String> {
    let mut response = String::new();
    let mut conn = BufReader::new(stream);
    let read_result = conn.read_line(&mut response);
    if read_result.is_ok() {
        let result = response;
        let file_list = result.split(",");
        let count = file_list.clone().count();
        println!("received vector: {:?} - count: {}", file_list, count);
        return file_list
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|item| item.to_string())
            .collect();
    }
    return Vec::new();
}

fn request_for_file_list(stream: &mut TcpStream) {
    let msg = format!("ll \r\n");
    let write_result = stream.write_all(msg.as_bytes());
    if write_result.is_err() {
        println!("error in writing response: {:?}", write_result.err());
    }
}

fn override_file(buffer: Vec<u8>, shared_directory: &String, fname: String) {
    let mut file = File::create(format!("{}/{}", shared_directory, fname)).unwrap();
    let write_result = file.write_all(&buffer);
    if write_result.is_err() {
        println!("error writing file: {:?}", write_result.err());
    }
}

fn extract_file_from_stream(file_size: u64, stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![0; file_size.try_into().unwrap()];
    let read_result = stream.read_exact(&mut buffer);
    if read_result.is_ok() {
        return buffer;
    }
    println!("Error reading file from strem: {:?}", read_result.err());
    return Vec::new();
}

fn send_data_request(stream: &mut TcpStream) {
    let write_result = stream.write("OK\r\n".as_bytes());
    if write_result.is_err() {
        println!(
            "error in trying to respond to server: {:?}",
            write_result.err()
        );
    }
}

fn calculate_file_to_save(file: &str, shared_directory: &String) -> (String, String) {
    let fname = Path::new(&file).file_name().unwrap().to_string_lossy();
    let file_to_save = format!("{}/{}", shared_directory, fname);
    (fname.to_string(), file_to_save)
}

fn size_sha2_request(stream: &TcpStream) -> Option<(u64, String)> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();
    let read_result = reader.read_line(&mut buffer);
    if read_result.is_err() {
        println!(
            "error in reading file (size, sha2): {:?}",
            read_result.err()
        );
        return None;
    }
    let buffer: Vec<String> = buffer
        .split(";")
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|item| item.to_string())
        .collect();
    let file_size: u64 = buffer.get(0).unwrap().trim().parse().unwrap();
    let file_hash = buffer.get(1).unwrap().trim();
    Some((file_size, file_hash.to_string()))
}

fn make_pull_request(file: &str, stream: &mut TcpStream) {
    let command = format!("pull;{}\r\n", file);
    let write_result = stream.write_all(command.as_bytes());
    if write_result.is_err() {
        println!("error in writing pull request: {:?}", write_result.err());
    }
}
