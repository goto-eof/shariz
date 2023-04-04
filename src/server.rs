use crate::{service::console_service::print_message, structures::config::Config};
use std::{
    io::{BufRead, BufReader, Stdout},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

pub async fn run_server(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) {
    let address = format!("{}:{}", "0.0.0.0", config.self_port);
    let listener = TcpListener::bind(address).unwrap();
    print_message(
        stdout_rw_lock.clone(),
        5,
        format!("server is listening on port: {}", config.self_port).as_str(),
    )
    .await;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                print_message(
                    stdout_rw_lock.clone(),
                    5,
                    format!("incomed new connection: {}", stream.peer_addr().unwrap()).as_str(),
                )
                .await;
                let stdout_rw_lock_clone = stdout_rw_lock.clone();
                tokio::spawn(async move { receive_data(stdout_rw_lock_clone, stream).await });
            }
            Err(e) => {
                print_message(
                    stdout_rw_lock.clone(),
                    5,
                    format!("connection failed: {}", e).as_str(),
                )
                .await;
            }
        }
    }
    drop(listener);
}

pub async fn receive_data(stdout_rw_lock: Arc<RwLock<Stdout>>, mut stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line.unwrap();
    }
}
