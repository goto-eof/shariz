use crate::client::run_client;
use crate::server::run_server;
use crate::util::home_util::print_header;
use crate::{service::console_service::print_message, structures::config::Config};
use config_file::FromConfigFile;
use std::{
    io::{stdout, Stdout},
    sync::{Arc, RwLock},
};
mod client;
mod server;
mod service;
mod structures;
mod util;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let config = Config::from_config_file("configuration/configuration.json").unwrap();

    let stdout_rw_lock: Arc<RwLock<Stdout>> = Arc::new(RwLock::new(stdout()));

    print_header(stdout_rw_lock.clone(), &config).await;

    let server = run_server(&config, stdout_rw_lock.clone());

    let client = run_client(&config, stdout_rw_lock.clone());

    let results = futures::future::join(client, server).await;
}
