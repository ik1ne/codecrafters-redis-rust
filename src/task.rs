use anyhow::Result;
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinSet;

use crate::resp::Resp;

pub async fn run(listener: TcpListener) -> Result<()> {
    let mut join_set = JoinSet::new();

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

        join_set.spawn(async move {
            let (read, write) = socket.split();

            handle_operation(&mut BufReader::new(read), write).await
        });
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
) -> Result<()> {
    let resp = Resp::parse(read).await?;

    resp.run(write).await?;

    Ok(())
}
