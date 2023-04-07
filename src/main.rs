use crate::client::run_client;
use crate::server::run_server;
use crate::{service::console_service::print_message, structures::config::Config};
use config_file::FromConfigFile;
use std::io::stdout;
use std::thread;
use std::time::Duration;
use util::home_util::print_header;
mod client;
mod server;
mod service;
mod structures;
mod util;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let config = Config::from_config_file("configuration/configuration.json").unwrap();

    print_header(&mut stdout(), &config).await;

    run_server(&config).await;

    loop {
        run_client(&config).await;
        thread::sleep(Duration::from_millis(10000));
    }
}
