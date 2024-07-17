use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

use task::{replication, serve_client};

use crate::config::{Config, Role};
use crate::storage::Storage;

mod config;
mod resp;
mod storage;
mod task;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(Config::parse_parameter(std::env::args().skip(1))?);
    let storage = Arc::new(RwLock::new(Storage::new(&config)));

    let mut join_set: JoinSet<Result<()>> = JoinSet::new();

    if let Role::Slave {
        master_host,
        master_port,
    } = &config.role
    {
        let replication_task = replication::start_replication(
            format!("{}:{}", master_host, master_port),
            Arc::clone(&storage),
            Arc::clone(&config),
        );

        join_set.spawn(replication_task);
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;

    join_set.spawn(serve_client::run(listener, storage));

    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(result) => {
                if let Err(e) = result {
                    eprintln!("error occurred; error = {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("join error occurred; error = {:?}", e);
            }
        }
    }

    Ok(())
}
