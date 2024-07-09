use anyhow::Result;
use tokio::net::TcpListener;

mod resp;
mod storage;
mod task;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    task::run(listener).await?;

    Ok(())
}
