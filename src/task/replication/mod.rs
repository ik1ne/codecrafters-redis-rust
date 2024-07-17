use std::sync::Arc;
use std::sync::RwLock;

use anyhow::{bail, Result};
use tokio::io::{AsyncBufRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::config::Config;
use crate::resp::{Array, Resp, SimpleString};
use crate::storage::Storage;

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

    handle_psync(&mut read, &mut write).await?;

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

async fn handle_psync(
    mut read: impl AsyncBufRead + Send + Unpin,
    mut write: impl AsyncWrite + Send + Unpin,
) -> Result<()> {
    let psync = Resp::Array(Array(vec![
        Resp::SimpleString(SimpleString("PSYNC".to_string())),
        Resp::SimpleString(SimpleString("?".to_string())),
        Resp::SimpleString(SimpleString("-1".to_string())),
    ]));
    let psync_string = psync.to_string();

    write.write_all(psync_string.as_bytes()).await?;

    let response = Resp::parse(&mut read).await?;

    println!(r#"response = "{}""#, response.plain_string()?);

    Ok(())
}
