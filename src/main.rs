use anyhow::Result;
use tokio::net::TcpListener;

mod config;
mod resp;
mod storage;
mod task;

#[tokio::main]
async fn main() -> Result<()> {
    let port = get_arg_value(&mut std::env::args(), "--port").unwrap_or("6379".to_string());

    let config = Default::default();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    task::run(listener, config).await?;

    Ok(())
}

fn get_arg_value(args: &mut std::env::Args, arg_name: &str) -> Option<String> {
    args.find(|arg| arg == arg_name).and_then(|_| args.next())
}
