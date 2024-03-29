use crate::dao::file_dao::{
    delete_file_db, list_all_files_on_db, update_file_delete_status, CREATED, DELETED,
};
use crate::service::file_service::calculate_file_hash;
use crate::service::processors::processor_local_update::LocalUpdateProcessor;
use crate::structures::config::Config;
use chrono::NaiveDateTime;
use core::panic;
use diesel::SqliteConnection;
use shariz::models::FileDB;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub async fn run_client(
    config: &Config,
    db_connection_mutex: Arc<Mutex<SqliteConnection>>,
) -> JoinHandle<bool> {
    let shared_directory = config.shared_directory.clone();
    let target_ip = config.target_ip.clone();
    let port = config.target_port;
    tokio::spawn(async move {
        let connection = enstablish_connection(target_ip, port);
        if connection.is_ok() {
            let stream = connection.unwrap();

            let cloned_stream = stream.try_clone();
            if cloned_stream.is_err() {
                panic!("client: failed to clone stream");
            }
            let mut cloned_stream = cloned_stream.unwrap();

            request_for_file_list(&mut cloned_stream);
            let server_file_list = extract_server_file_list(&stream);
            let all_db_files =
                refresh_and_retrieve_all_db_files(&db_connection_mutex, &shared_directory);

            for file_on_server in server_file_list {
                if file_on_server.0.trim().len() > 0 {
                    let file_on_db = all_db_files
                        .iter()
                        .find(|file_db| file_db.name.eq(&file_on_server.0));
                    let file_path = format!("{}/{}", &shared_directory, file_on_server.0.trim());

                    if file_on_db.is_some() {
                        let file_on_db = file_on_db.unwrap();
                        let file_db_last_update = file_on_db.last_update.unwrap();

                        if file_on_server.1 == DELETED
                            && file_on_db.status == CREATED
                            && file_db_last_update.le(&file_on_server.2)
                        {
                            println!("client: case: deleted on server, not deleted on client");
                            file_delete_and_update_status(
                                &file_path,
                                file_on_db,
                                &file_on_server,
                                &db_connection_mutex,
                            );
                        } else if file_on_server.1 == CREATED
                            && file_on_db.status == DELETED
                            && file_on_server.2.gt(&file_db_last_update)
                        {
                            println!(
                                "client: case: not deleted on server, deleted on client before"
                            );
                            process_file(
                                file_on_server,
                                &mut cloned_stream,
                                &stream,
                                &shared_directory,
                            );
                        } else if file_on_server.1 == CREATED
                            && file_on_db.status == CREATED
                            && !file_on_server.3.eq(&file_on_db.sha2)
                            && file_db_last_update.ge(&file_on_server.2)
                        {
                            println!(
                                "client: case: file corruption: {}!={}",
                                file_on_server.3, file_on_db.sha2
                            );
                            process_file(
                                file_on_server,
                                &mut cloned_stream,
                                &stream,
                                &shared_directory,
                            );
                        } else {
                            // println!("client: deleted on client: {} - deleted on server: {} - last update on client: {} - last update on server: {}", file_on_db.status, file_on_server.1, file_db_last_update, file_on_server.2);
                        }
                    } else {
                        if file_on_server.1 == CREATED {
                            println!("client: downloading file...");
                            process_file(
                                file_on_server,
                                &mut cloned_stream,
                                &stream,
                                &shared_directory,
                            );
                        } else {
                            //println!("client: file alredy sync");
                        }
                    }
                }
            }

            let all_db_files =
                refresh_and_retrieve_all_db_files(&db_connection_mutex, &shared_directory);

            all_db_files.iter().for_each(|file_db| {
                if file_db.status == DELETED {
                    request_for_del_record(&mut cloned_stream, &file_db.name);
                    let result_del_record = extract_server_del_record_response(&mut cloned_stream);
                    if result_del_record {
                        println!("client: server deleted file successfully!");
                        if delete_file_db(&mut db_connection_mutex.lock().unwrap(), &file_db.name) {
                            println!("client: also client deleted record");
                        } else {
                            println!("client: ERROR deleting record");
                        }
                    } else {
                        println!("client: ERROR server did not deleted file");
                    }
                }
            });

            let result_shutdown = stream.shutdown(Shutdown::Both);
            if result_shutdown.is_err() {
                println!("client: shutdown error: {:?}", result_shutdown.err());
            }
            sleep(Duration::from_millis(10000)).await;
            return true;
        } else {
            println!("client: connection error: {:?}", connection.err());
            sleep(Duration::from_millis(10000)).await;
            return true;
        }
    })
}

fn request_for_del_record(stream: &mut TcpStream, fname: &str) {
    let msg = format!("del;{}\r\n", fname);
    let write_result = stream.write_all(msg.as_bytes());
    if write_result.is_err() {
        println!("client: error in writing request: {:?}", write_result.err());
    }
}

pub fn extract_server_del_record_response(stream: &mut TcpStream) -> bool {
    let mut response = String::new();
    let mut buf_reader = BufReader::new(stream);
    let read_result = buf_reader.read_line(&mut response);
    if read_result.is_ok() && response.starts_with("OK") {
        return true;
    }
    return false;
}

fn enstablish_connection(address: String, port: u16) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(format!("{}:{}", address, port))
}

fn file_delete_and_update_status(
    file_path: &String,
    file_on_db: &FileDB,
    file_on_server: &(String, i32, NaiveDateTime, String),
    db_connection_mutex: &Arc<Mutex<SqliteConnection>>,
) {
    if Path::new(file_path).exists() {
        println!(
            "client: =====> case 1 - dbfile: {:?} - {:?}",
            &file_on_db, file_on_server
        );

        fs::remove_file(file_path).unwrap();
        update_file_delete_status(
            &mut db_connection_mutex.lock().unwrap(),
            file_on_server.0.trim().to_owned(),
            1,
        );
    }
}

fn refresh_and_retrieve_all_db_files(
    db_connection_mutex: &Arc<Mutex<SqliteConnection>>,
    shared_directory: &str,
) -> Vec<FileDB> {
    let connection = db_connection_mutex.lock();
    if connection.is_err() {
        panic!("client: unable to open db connection");
    }
    let mut connection = connection.unwrap();
    let result = LocalUpdateProcessor::sync_disk_with_db(&mut connection, shared_directory);
    if !result {
        panic!("client: unable to sync disk with db");
    }
    list_all_files_on_db(&mut connection)
}

fn process_file(
    file: (String, i32, NaiveDateTime, String),
    cloned_stream: &mut TcpStream,
    stream: &TcpStream,
    shared_directory: &String,
) {
    println!("client: make a pull request...");
    make_pull_request(file.0.as_str(), cloned_stream);

    let opt_file_size_hash = size_sha2_request(stream);
    println!(
        "client: received size and file hash: {:?}",
        opt_file_size_hash
    );
    if opt_file_size_hash.is_none() {
        println!("client: sending ko (1)");
        send_ko(cloned_stream);
        return;
    }
    let (file_size, file_hash) = opt_file_size_hash.unwrap();
    let (fname, file_to_save) = calculate_file_to_save(file.0.as_str(), shared_directory);
    println!("client: I'll save the file {} in {}", fname, file_to_save);
    if !Path::new(&file_to_save).exists()
        || Path::new(&file_to_save).exists()
            && file_size != fs::metadata(&file_to_save).unwrap().len()
        || calculate_file_hash(&file_to_save).unwrap() != file_hash
    {
        println!("client: requesting file to server...");
        send_data_request(cloned_stream);
        println!("client: extracting data from the stream....");
        let buffer = extract_file_from_stream(file_size, cloned_stream);
        println!("client: writing file...");
        override_file(buffer, shared_directory, fname);
    } else {
        send_ko(cloned_stream);
    }
}

fn send_ko(cloned_stream: &mut TcpStream) {
    let write_result = cloned_stream.write("KO\r\n".as_bytes());
    println!("client: sent KO to serve");
    if write_result.is_err() {
        println!("client: can't send respond to server: {:?}", write_result);
    }
}

fn extract_server_file_list(stream: &TcpStream) -> Vec<(String, i32, NaiveDateTime, String)> {
    let mut response = String::new();
    let mut buf_reader = BufReader::new(stream);
    let read_result = buf_reader.read_line(&mut response);
    if read_result.is_ok() {
        let result = response;
        let file_list = result.split(",");
        let file_list_vec = file_list.collect::<Vec<&str>>();
        println!("client: result of ll command: {}", file_list_vec.len());
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
                    NaiveDateTime::from_timestamp_millis(vector.get(2).unwrap().parse().unwrap())
                        .unwrap(),
                    vector.get(3).unwrap().to_string(),
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
        println!(
            "client: error in writing response: {:?}",
            write_result.err()
        );
    }
}

fn override_file(buffer: Vec<u8>, shared_directory: &String, fname: String) {
    let file_path = format!("{}/{}", shared_directory, fname);
    let file_tmp_path = format!("{}/{}.shariz", shared_directory, fname);
    println!("client: writing on file: {}", file_tmp_path);
    let mut file = File::create(&file_tmp_path).unwrap();
    let write_result = file.write_all(&buffer);
    if write_result.is_err() {
        println!("client: error writing file: {:?}", write_result.err());
    } else {
        let file_rename_result = fs::rename(file_tmp_path, file_path);
        if file_rename_result.is_err() {
            println!(
                "client: error renaming file: {:?}",
                file_rename_result.err()
            );
        }
    }
}

pub fn extract_file_from_stream(file_size: u64, stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![0; file_size.try_into().unwrap()];
    let read_result = stream.read_exact(&mut buffer);
    if read_result.is_ok() {
        return buffer;
    }
    println!(
        "client: error reading file from strem: {:?}",
        read_result.err()
    );
    return Vec::new();
}

fn send_data_request(stream: &mut TcpStream) {
    let write_result = stream.write("OK\r\n".as_bytes());
    if write_result.is_err() {
        println!(
            "client: error in trying to respond to server: {:?}",
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
            "client: error in reading file (size, sha2): {:?}",
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
    let file_size_result = buffer.get(0).unwrap().trim().parse();
    if file_size_result.is_err() {
        println!("clinet: invalid file size received from server");
        return None;
    }
    let file_size: u64 = file_size_result.unwrap();
    let file_hash = buffer.get(1).unwrap().trim();
    Some((file_size, file_hash.to_string()))
}

fn make_pull_request(file: &str, stream: &mut TcpStream) {
    let command = format!("pull;{}\r\n", file);
    let write_result = stream.write_all(command.as_bytes());
    if write_result.is_err() {
        println!(
            "client: error in writing pull request: {:?}",
            write_result.err()
        );
    }
}
