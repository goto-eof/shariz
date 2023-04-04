use std::{
    io::{stdout, Stdout},
    sync::{Arc, RwLock},
};

use crate::util::home_util::print_header;
use crate::{
    service::console_service::{clear_console, print_message},
    structures::config::Config,
};
use config_file::FromConfigFile;
mod service;
mod structures;
mod util;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let config = Config::from_config_file("configuration/configuration.json").unwrap();

    let stdout_rw_lock: Arc<RwLock<Stdout>> = Arc::new(RwLock::new(stdout()));

    print_header(stdout_rw_lock.clone(), &config).await;
}
