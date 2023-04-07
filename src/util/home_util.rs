use crate::structures::config::Config;
use crate::{print_message, service::console_service::clear_console};
use std::io::Stdout;

pub async fn print_header(stdout: &mut Stdout, config: &Config) {
    clear_console(stdout).await;

    print_message(stdout, 0, r#"   ______            _   "#).await;
    print_message(stdout, 1, r#"  / __/ /  ___ _____(_)__"#).await;
    print_message(stdout, 2, r#" _\ \/ _ \/ _ `/ __/ /_ /"#).await;
    print_message(stdout, 3, r#"/___/_//_/\_,_/_/ /_//__/"#).await;
    print_message(stdout, 4, r#"                         "#).await;

    print_message(
        stdout,
        5,
        format!("target: {}:{}", config.target_ip, config.target_port).as_str(),
    )
    .await;

    print_message(
        stdout,
        6,
        format!("self port: {}", config.self_port).as_str(),
    )
    .await;

    print_message(
        stdout,
        7,
        format!("shared directory: {}", config.shared_directory).as_str(),
    )
    .await;
}
