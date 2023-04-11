use crate::service::db_service::{list_all_files, update_file_delete_status};
use crate::service::file_service::{calculate_file_hash, extract_fname};
use crate::structures::config::Config;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use rusqlite::Connection;
use tokio::task::JoinHandle;

pub async fn run_client(
    config: &Config,
    db_connection_mutex: Arc<Mutex<Connection>>,
) -> JoinHandle<()> {
    // let address = format!("{}:{}", config.target_ip, config.target_port);
    let shared_directory = config.shared_directory.clone();
    let target_ip = config.target_ip.clone();
    // TODO delete the row bellow: this is for testing purposes
    // let shared_directory = format!("{}/{}", shared_directory, "tmp");
    let rd_timeout = config.client_rd_timeout;
    let wr_timeout = config.client_wr_timeout;
    let port = config.target_port;
    tokio::spawn(async move {
        let connection = discover_service(target_ip, port);
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

            let files_result = fs::read_dir(&shared_directory).unwrap();
            let mut files_on_disk: Vec<String> = vec![];
            for file_result in files_result {
                if file_result.is_err() {
                    println!("unable to list file");
                }
                let file = file_result.unwrap();
                let file_name = extract_fname(&file.path().to_string_lossy().to_string());
                files_on_disk.push(file_name);
            }
            let file_list = read_file_list(&stream);
            let all_db_files =
                refresh_and_retrieve_all_db_files(&db_connection_mutex, files_on_disk);

            for file_on_server in file_list {
                if file_on_server.0.trim().len() > 0 {
                    let file_on_db = all_db_files
                        .iter()
                        .find(|file_db| file_db.name.eq(&file_on_server.0));
                    let file_path = format!("{}/{}", &shared_directory, file_on_server.0.trim());

                    if file_on_db.is_some() {
                        let file_on_db = file_on_db.unwrap();

                        if file_on_server.1 == 1
                            && file_on_db.status == 0
                            && file_on_db.last_update.le(&file_on_server.2)
                        {
                            println!("case: deleted on server, not deleted on client");
                            file_delete_and_update_status(
                                &file_path,
                                file_on_db,
                                &file_on_server,
                                &db_connection_mutex,
                            );
                        } else if file_on_server.1 == 0
                            && file_on_db.status == 1
                            && file_on_server.2.gt(&file_on_db.last_update)
                        {
                            println!("case: not deleted on server, deleted on client");
                            process_file(
                                file_on_server,
                                &mut cloned_stream,
                                &stream,
                                &shared_directory,
                            );
                        }
                    } else {
                        if file_on_server.1 == 0 {
                            process_file(
                                file_on_server,
                                &mut cloned_stream,
                                &stream,
                                &shared_directory,
                            );
                        }
                    }
                }
            }
            let result_shutdown = stream.shutdown(Shutdown::Both);
            if result_shutdown.is_err() {
                println!("shutdown error");
            }
            return ();
        } else {
            println!("connection error: {:?}", connection.err());
            return ();
        }
    })
}

fn discover_service(address: String, port: u16) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(format!("{}:{}", address, port))
    // let mut i = 0;
    // let mut address = make_address(i, port);
    // let mut connection = async_std::net::TcpStream::connect(&address).await;
    // let mut connection = Err(std::io::Error::new(ErrorKind::Other, "oh no!"));
    // let my_local_ip = local_ip().unwrap().to_string();
    // while connection.is_err() {
    //     if format!("192.168.1.{}", i).eq(&my_local_ip) {
    //         i = i + 1;
    //         address = make_address(i, port);
    //     }
    //     println!("conn: {:?}", connection);
    //     i = i + 1;
    //     println!("yabado");
    //     address = make_address(i, port);
    //     if i > 255 {
    //         i = 0;
    //     }
    //     println!("searching for server: {}", &address);
    //     connection = TcpStream::connect(address.clone());
    // }
    // println!("connected to: {} - localip: {}", &address, my_local_ip);
    // Ok(TcpStream::connect(address).unwrap())
}

fn make_address(i: i32, port: u16) -> String {
    let mut address = format!("192.168.1.{}:{}", i, port);
    // let mut address = format!("192.168.1.{}", i);
    address
}

fn file_delete_and_update_status(
    file_path: &String,
    file_on_db: &crate::structures::file::DbFile,
    file_on_server: &(String, i32, DateTime<FixedOffset>),
    db_connection_mutex: &Arc<Mutex<Connection>>,
) {
    if Path::new(file_path).exists() {
        println!(
            "=====> case 1 - dbfile: {:?} - {:?}",
            &file_on_db, file_on_server
        );

        fs::remove_file(file_path).unwrap();
        update_file_delete_status(
            &db_connection_mutex.lock().unwrap(),
            file_on_server.0.trim().to_owned(),
            1,
        );
    }
}

fn refresh_and_retrieve_all_db_files(
    db_connection_mutex: &Arc<Mutex<Connection>>,
    files_on_disk: Vec<String>,
) -> Vec<crate::structures::file::DbFile> {
    let all_db_files = list_all_files(&db_connection_mutex.lock().unwrap()).unwrap();
    all_db_files.iter().for_each(|file_on_db| {
        if !files_on_disk.contains(&file_on_db.name) {
            if file_on_db.status != 1 {
                println!("----> delete {}", &file_on_db.name);
                update_file_delete_status(
                    &db_connection_mutex.lock().unwrap(),
                    (&file_on_db.name).to_string(),
                    1,
                );
            }
        } else {
            if file_on_db.status != 0 {
                println!("----> undelete {}", &file_on_db.name);
                update_file_delete_status(
                    &db_connection_mutex.lock().unwrap(),
                    (&file_on_db.name).to_string(),
                    0,
                );
            }
        }
    });
    let all_db_files = list_all_files(&db_connection_mutex.lock().unwrap()).unwrap();
    all_db_files
}

fn process_file(
    file: (String, i32, DateTime<FixedOffset>),
    cloned_stream: &mut TcpStream,
    stream: &TcpStream,
    shared_directory: &String,
) {
    make_pull_request(file.0.as_str(), cloned_stream);

    let opt_file_size_hash = size_sha2_request(stream);
    if opt_file_size_hash.is_none() {
        println!("sending ko (1)");
        send_ko(cloned_stream);
    }
    let (file_size, file_hash) = opt_file_size_hash.unwrap();
    let (fname, file_to_save) = calculate_file_to_save(file.0.as_str(), shared_directory);

    if !Path::new(&file_to_save).exists()
        || Path::new(&file_to_save).exists()
            && file_size != fs::metadata(&file_to_save).unwrap().len()
        || calculate_file_hash(&file_to_save).unwrap() != file_hash
    {
        send_data_request(cloned_stream);
        let buffer = extract_file_from_stream(file_size, cloned_stream);
        override_file(buffer, shared_directory, fname);
    } else {
        send_ko(cloned_stream);
    }
}

fn send_ko(cloned_stream: &mut TcpStream) {
    let write_result = cloned_stream.write("KO\r\n".as_bytes());
    if write_result.is_err() {
        println!("can't send respond to server: {:?}", write_result);
    }
}

fn read_file_list(stream: &TcpStream) -> Vec<(String, i32, DateTime<FixedOffset>)> {
    let mut response = String::new();
    let mut conn = BufReader::new(stream);
    let read_result = conn.read_line(&mut response);
    if read_result.is_ok() {
        let result = response;
        let file_list = result.split(",");
        let file_list_vec = file_list.collect::<Vec<&str>>();
        if file_list_vec.len() == 1 && !file_list_vec.get(0).unwrap().contains(";") {
            return Vec::new();
        }
        return file_list_vec
            .into_iter()
            .filter(|item| item.contains(";"))
            .map(|item| {
                let vector = item.split(";").collect::<Vec<&str>>();
                return (
                    vector.get(0).unwrap().to_string(),
                    vector.get(1).unwrap().parse::<i32>().unwrap(),
                    DateTime::parse_from_rfc3339(vector.get(2).unwrap()).unwrap(),
                );
            })
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
    let file_path = format!("{}/{}", shared_directory, fname);
    println!("writing on file: {}", file_path);
    let mut file = File::create(file_path).unwrap();
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
