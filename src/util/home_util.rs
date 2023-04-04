use crate::print_message;
use crate::{service::console_service::clear_console, structures::config::Config};
use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

pub async fn print_header(stdout_rw_lock: Arc<RwLock<Stdout>>, config: &Config) {
    clear_console(stdout_rw_lock.clone()).await;

    print_message(stdout_rw_lock.clone(), 0, r#"   ______            _   "#).await;
    print_message(stdout_rw_lock.clone(), 1, r#"  / __/ /  ___ _____(_)__"#).await;
    print_message(stdout_rw_lock.clone(), 2, r#" _\ \/ _ \/ _ `/ __/ /_ /"#).await;
    print_message(stdout_rw_lock.clone(), 3, r#"/___/_//_/\_,_/_/ /_//__/"#).await;
    print_message(stdout_rw_lock.clone(), 4, r#"                         "#).await;

    print_message(
        stdout_rw_lock.clone(),
        5,
        format!("target: {}:{}", config.target_ip, config.target_port).as_str(),
    )
    .await;

    print_message(
        stdout_rw_lock.clone(),
        6,
        format!("self port: {}", config.self_port).as_str(),
    )
    .await;

    print_message(
        stdout_rw_lock.clone(),
        7,
        format!("shared directory: {}", config.shared_directory).as_str(),
    )
    .await;


}
