use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub target_ip: String,
    pub target_port: u16,
    pub self_port: u16,
    pub shared_directory: String,
}
