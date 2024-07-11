use std::sync::{Arc, RwLock};

use anyhow::Result;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::storage::Storage;
mod config;
mod resp;
mod storage;
mod task;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse_parameter(std::env::args().skip(1))?;

    let storage = Arc::new(RwLock::new(Storage::new(&config)));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;

    task::run(listener, storage).await?;

    Ok(())
}
