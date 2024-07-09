use std::sync::{Arc, RwLock};

use anyhow::Result;
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinSet;

use crate::resp::Resp;
use crate::storage::Storage;

pub async fn run(listener: TcpListener) -> Result<()> {
    let mut join_set: JoinSet<Result<()>> = JoinSet::new();
    let storage = Arc::new(RwLock::new(Storage::new()));

    loop {
        let (mut socket, _addr) = match listener.accept().await {
            Ok(socket_addr) => socket_addr,
            Err(e) => {
                eprintln!(
                    "an error occurred while accepting a connection; error = {:?}",
                    e
                );
                break;
            }
        };
        {
            let storage = Arc::clone(&storage);
            join_set.spawn(async move {
                let (read, mut write) = socket.split();
                let mut buf_reader = BufReader::new(read);
                loop {
                    handle_operation(&mut buf_reader, &mut write, storage.clone()).await?;
                }
            });
        }
    }

    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            eprintln!("an error occurred; error = {:?}", e);
        }
    }

    println!("shutting down");

    Ok(())
}

async fn handle_operation(
    read: &mut (impl AsyncBufRead + Unpin + Send),
    write: impl AsyncWrite + Unpin,
    storage: Arc<RwLock<Storage>>,
) -> Result<()> {
    let resp = Resp::parse(read).await?;

    resp.run(write, storage).await?;

    Ok(())
}
