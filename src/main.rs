use crate::config::Config;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
mod config;
mod resp;
mod storage;
mod task;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(Config::parse_args()?);

    let storage = Default::default();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;

    task::run(listener, config, storage).await?;

    Ok(())
}
