use std::sync::{Arc, Mutex, RwLock};

use anyhow::{anyhow, Result};
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::resp::Resp;
use crate::storage::Storage;

pub async fn run(listener: TcpListener, storage: Arc<RwLock<Storage>>) -> Result<()> {
    let join_set: Arc<Mutex<JoinSet<Result<()>>>> = Arc::new(Mutex::new(JoinSet::new()));

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut shutdown_rx_for_listener = shutdown_rx.clone();

    let shutdown_rx_task = shutdown_rx_for_listener.changed();
    tokio::pin!(shutdown_rx_task);

    let listener_loop_task = listener_loop(listener, storage, Arc::clone(&join_set), shutdown_rx);

    tokio::select! {
        _ = &mut shutdown_rx_task => {}
        loop_result = listener_loop_task => {
            if loop_result.is_err() {
                eprintln!("listener loop error occurred; error = {:?}", loop_result.err());
            }
            shutdown_tx.send(())?
        }
    }

    let mut join_set = Arc::try_unwrap(join_set)
        .map_err(|_| anyhow!("unable to unwrap join set; other references exist"))?
        .into_inner()?;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(inner_result) => {
                if let Err(e) = inner_result {
                    eprintln!("error occurred; error = {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("join error occurred; error = {:?}", e);
            }
        }
    }

    println!("shutting down");

    Ok(())
}

async fn listener_loop(
    listener: TcpListener,
    storage: Arc<RwLock<Storage>>,
    join_set: Arc<Mutex<JoinSet<Result<()>>>>,
    shutdown_rx: watch::Receiver<()>,
) -> Result<()> {
    loop {
        let (mut stream, _address) = listener.accept().await?;

        let storage = Arc::clone(&storage);
        let mut join_set = join_set
            .lock()
            .map_err(|_| anyhow!("unable to lock join set"))?;
        let mut shutdown_rx = shutdown_rx.clone();

        join_set.spawn(async move {
            let (read, write) = stream.split();
            let mut read = BufReader::new(read);

            let resp_loop = run_resp_loop(&mut read, write, storage);

            tokio::select! {
                _ = shutdown_rx.changed() => {}
                result = resp_loop => {
                    if result.is_err() {
                        eprintln!("resp loop error occurred; error = {:?}", result.err());
                    }
                }
            }

            Ok(())
        });
    }
}

async fn run_resp_loop(
    read: &mut (impl AsyncBufRead + Unpin + Send),
    mut write: impl AsyncWrite + Unpin,
    storage: Arc<RwLock<Storage>>,
) -> Result<()> {
    loop {
        let resp = Resp::parse(read).await?;

        resp.run(&mut write, Arc::clone(&storage)).await?;
    }
}
