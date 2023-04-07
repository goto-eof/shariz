use crate::{
    service::{
        console_service::print_message,
        processors::{process_pull::PullProcessor, processor_ll::LLProcessor},
    },
    structures::{command_processor::CommandProcessorType, config::Config},
};
use std::{
    io::{BufRead, BufReader, Stdout},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, RwLock},
};

pub async fn run_server(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) {
    let port = config.self_port;
    let processors: Arc<Mutex<Vec<CommandProcessorType>>> =
        Arc::new(Mutex::new(prepare_command_processors(config)));
    tokio::spawn(async move {
        let address = format!("{}:{}", "0.0.0.0", port);
        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            let clone = processors.clone();
            match stream {
                Ok(mut stream) => {
                    let stdout_rw_lock_clone = stdout_rw_lock.clone();
                    tokio::spawn(async move {
                        receive_data(stdout_rw_lock_clone, &mut stream, clone).await
                    })
                    .await;
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

pub async fn receive_data(
    stdout_rw_lock: Arc<RwLock<Stdout>>,
    stream: &mut TcpStream,
    processors: Arc<Mutex<Vec<CommandProcessorType>>>,
) {
    let mut stream_clone = stream.try_clone().unwrap();
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        println!("{}", format!("server received: {:?}", line));

        if line.is_ok() {
            let line = line.unwrap();
            for processor in processors.lock().unwrap().iter() {
                if processor.accept(&line) {
                    processor.process(&line, &mut stream_clone);
                    break;
                }
            }
        }
    }
}

pub fn prepare_command_processors(config: &Config) -> Vec<CommandProcessorType> {
    let mut processors: Vec<CommandProcessorType> = Vec::new();
    processors.push(Box::new(LLProcessor {
        search_directory: config.shared_directory.to_owned(),
    }));
    processors.push(Box::new(PullProcessor {
        search_directory: config.shared_directory.to_owned(),
    }));
    return processors;
}
