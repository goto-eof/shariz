use crate::structures::config::Config;
use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

pub async fn run_client(config: &Config, stdout_rw_lock: Arc<RwLock<Stdout>>) {}
