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

pub async fn run_server(config: &Config) {
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
                    let result_spawn =
                        tokio::spawn(async move { receive_data(&mut stream, clone).await }).await;
                    if result_spawn.is_err() {
                        println!("error spawning handler: {:?}", result_spawn.err());
                    }
                }
                Err(e) => {
                    println!("stream error: {:?}", e);
                }
            }
        }
        drop(listener);
    });
}

pub async fn receive_data(
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
            let processors_result = processors.lock();
            if processors_result.is_err() {
                println!("processors error: {:?}", processors_result.err());
            } else {
                let processors = processors_result.unwrap();
                for processor in processors.iter() {
                    if processor.accept(&line) {
                        let operation_result = processor.process(&line, &mut stream_clone);
                        if !operation_result {
                            println!("failed to execute the `{}` operation", &line);
                        }
                        break;
                    }
                }
            }
        }
    }
}

pub fn prepare_command_processors(config: &Config) -> Vec<CommandProcessorType> {
    let mut processors: Vec<CommandProcessorType> = Vec::new();
    let shared_directory = config.shared_directory.as_str();
    processors.push(Box::new(LLProcessor::new(shared_directory)));
    processors.push(Box::new(PullProcessor::new(shared_directory)));
    return processors;
}
