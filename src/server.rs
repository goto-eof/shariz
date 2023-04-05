use crate::{
    service::{
        console_service::print_message,
        processors::{process_pull::PullProcessor, processor_ll::LLProcessor},
    },
    structures::{command_processor::CommandProcessorType, config::Config},
};
use std::{
    io::{BufRead, BufReader, Stdout, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

pub async fn run_server(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) {
    let port = config.self_port;
    tokio::spawn(async move {
        let address = format!("{}:{}", "0.0.0.0", port);
        let listener = TcpListener::bind(address).unwrap();
        print_message(
            stdout_rw_lock.clone(),
            5,
            format!("server is listening on port: {}", port).as_str(),
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
                    tokio::spawn(async move { receive_data(stdout_rw_lock_clone, stream).await })
                        .await
                        .unwrap();
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
    });
}

pub async fn receive_data(stdout_rw_lock: Arc<RwLock<Stdout>>, mut stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line.unwrap();

        print_message(
            stdout_rw_lock.clone(),
            5,
            format!("server received: {}", line).as_str(),
        )
        .await;
        let mut stream = &stream;
        stream.write("halo".as_bytes()).unwrap();
    }
}

pub fn prepare_command_processors() -> Vec<CommandProcessorType> {
    let mut processors: Vec<CommandProcessorType> = Vec::new();
    processors.push(Box::new(LLProcessor {}));
    processors.push(Box::new(PullProcessor {}));
    return processors;
}
