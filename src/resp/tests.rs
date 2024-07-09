use anyhow::Result;
use std::sync::{Arc, RwLock};
use tokio::io::BufReader;

use crate::resp::Resp;
use crate::storage::Storage;

pub async fn assert_parse(input: &str, expected: Resp) -> Result<()> {
    let mut buf = BufReader::new(input.as_bytes());
    let actual = Resp::parse(&mut buf).await?;

    assert_eq!(actual, expected);

    assert_eq!(buf.buffer().len(), 0, "buffer should be empty");

    Ok(())
}

pub async fn assert_run(input: Resp, expected: Resp) -> Result<()> {
    let mut buf = Vec::new();
    input
        .run(&mut buf, Arc::new(RwLock::new(Storage::new())))
        .await?;

    let result = String::from_utf8(buf)?;

    assert_eq!(result, expected.to_string());

    Ok(())
}

pub async fn assert_run_with_storage(
    input: Resp,
    expected: Resp,
    storage: Arc<RwLock<Storage>>,
) -> Result<()> {
    let mut buf = Vec::new();
    input.run(&mut buf, storage).await?;

    let result = String::from_utf8(buf)?;

    assert_eq!(result, expected.to_string());

    Ok(())
}
