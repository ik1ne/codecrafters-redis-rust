use std::sync::{Arc, Mutex, RwLock};

use anyhow::{anyhow, bail, Result};
use tokio::io::{AsyncBufRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::config::Config;
use crate::resp::{Array, Resp, SimpleString};
use crate::storage::Storage;

pub async fn serve_client(listener: TcpListener, storage: Arc<RwLock<Storage>>) -> Result<()> {
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

pub async fn start_replication(
    connection_string: String,
    _storage: Arc<RwLock<Storage>>,
    config: Arc<Config>,
) -> Result<()> {
    let mut sock = TcpStream::connect(connection_string).await?;

    let (read, mut write) = sock.split();

    let mut read = BufReader::new(read);

    send_ping_receive_pong(&mut read, &mut write).await?;

    send_replconf_receive_ok(&mut read, &mut write, &config).await?;

    Ok(())
}

async fn send_ping_receive_pong(
    mut read: impl AsyncBufRead + Send + Unpin,
    mut write: impl AsyncWrite + Send + Unpin,
) -> Result<()> {
    let ping = Resp::Array(Array(vec![Resp::SimpleString(SimpleString(
        "PING".to_string(),
    ))]));
    let ping_string = ping.to_string();

    write.write_all(ping_string.as_bytes()).await?;
    let response = Resp::parse(&mut read).await?;
    if response.plain_string()? != "PONG" {
        bail!("expected PONG, got {:?}", response);
    }

    Ok(())
}

async fn send_replconf_receive_ok(
    mut read: impl AsyncBufRead + Send + Unpin,
    mut write: impl AsyncWrite + Send + Unpin,
    config: &Config,
) -> Result<()> {
    let replconf = Resp::Array(Array(vec![
        Resp::SimpleString(SimpleString("REPLCONF".to_string())),
        Resp::SimpleString(SimpleString("listening-port".to_string())),
        Resp::SimpleString(SimpleString(config.port.to_string())),
    ]));
    let replconf_string = replconf.to_string();

    write.write_all(replconf_string.as_bytes()).await?;
    let response = Resp::parse(&mut read).await?;
    if response.plain_string()? != "OK" {
        bail!("expected OK, got {:?}", response);
    }

    let replconf = Resp::Array(Array(vec![
        Resp::SimpleString(SimpleString("REPLCONF".to_string())),
        Resp::SimpleString(SimpleString("capa".to_string())),
        Resp::SimpleString(SimpleString("psync2".to_string())),
    ]));

    let replconf_string = replconf.to_string();

    write.write_all(replconf_string.as_bytes()).await?;
    let response = Resp::parse(&mut read).await?;
    if response.plain_string()? != "OK" {
        bail!("expected OK, got {:?}", response);
    }

    Ok(())
}
