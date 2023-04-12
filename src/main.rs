use crate::client::run_client;
use crate::server::run_server;
use crate::{service::console_service::print_message, structures::config::Config};
use config_file::FromConfigFile;
use diesel::SqliteConnection;
use shariz::establish_connection;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use util::home_util::print_header;
mod client;
mod dao;
mod server;
mod service;
mod structures;
mod util;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let config = Config::from_config_file("configurations/configuration.json").unwrap();

    print_header(&mut stdout(), &config).await;

    // let db_connection = initialize_db();
    let db_connection = establish_connection();
    let db_connection_mutex: Arc<Mutex<SqliteConnection>> = Arc::new(Mutex::new(db_connection));

    run_server(&config, db_connection_mutex.clone()).await;

    loop {
        run_client(&config, db_connection_mutex.clone()).await;
        thread::sleep(Duration::from_millis(10000));
    }
}
