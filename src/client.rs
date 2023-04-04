use tokio::task::JoinHandle;

use crate::{service::console_service::print_message, structures::config::Config};
use std::{
    io::{Read, Stdout, Write},
    net::TcpStream,
    str::from_utf8,
    sync::{Arc, RwLock},
};

pub async fn run_client(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) -> JoinHandle<()> {
    let address = format!("{}:{}", config.target_ip, config.target_port);
    let stdout_rw_lock = stdout_rw_lock.clone();
    let port = config.target_port;
    tokio::spawn(async move {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                print_message(
                    stdout_rw_lock.clone(),
                    6,
                    format!("connected successfully to port: {}", port).as_str(),
                )
                .await;

                let msg = b"Hello World!\r\n";

                stream.write(msg).unwrap();

                print_message(stdout_rw_lock.clone(), 6, format!("Message sent").as_str()).await;

                let mut data = [0 as u8; 14]; // using 6 byte buffer
                match stream.read_exact(&mut data) {
                    Ok(_) => {
                        if &data == msg {
                            // OK!
                        } else {
                            let text = from_utf8(&data).unwrap();
                        }
                    }
                    Err(e) => {
                        // ERROR
                    }
                }
            }
            Err(e) => {
                // ERROR
            }
        }
    })
}
