use anyhow::Result;
use tokio::io::BufReader;

use crate::resp::Resp;

pub async fn assert_parse(input: &str, expected: Resp) -> Result<()> {
    let mut buf = BufReader::new(input.as_bytes());
    let actual = Resp::parse(&mut buf).await?;

    assert_eq!(actual, expected);

    assert_eq!(buf.buffer().len(), 0, "buffer should be empty");

    Ok(())
}
