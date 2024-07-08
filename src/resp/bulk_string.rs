use anyhow::{bail, Result};
use tokio::io::AsyncBufRead;

use crate::resp::{AsyncCrlfReadExt, RespParsable};

/// Represents a RESP bulk string.
///
/// Bulk string of NULL value is represented as `BulkString(None)`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BulkString(pub Option<String>);

impl RespParsable for BulkString {
    const PREFIX: char = '$';

    async fn parse_body(mut read: impl AsyncBufRead + Unpin + Send) -> Result<Self> {
        let num_bytes = read.read_crlf_line().await?.parse::<i128>()?;
        if num_bytes < 0 {
            return Ok(BulkString(None));
        }
        if num_bytes > u64::MAX as i128 {
            bail!("num_bytes too large: {}", num_bytes);
        }

        let bytes = read.read_bytes(num_bytes as u64).await?;

        read.read_crlf_line().await?;

        let line = String::from_utf8(bytes)?;

        Ok(BulkString(Some(line)))
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::tests::assert_parse;
    use crate::resp::Resp;

    use super::*;

    #[tokio::test]
    async fn test_parse_bulk_string() -> Result<()> {
        assert_parse(
            "$11\r\nhello world\r\n",
            Resp::BulkString(BulkString(Some("hello world".to_string()))),
        )
        .await
    }

    #[tokio::test]
    async fn test_parse_empty_bulk_string() -> Result<()> {
        assert_parse(
            "$0\r\n\r\n",
            Resp::BulkString(BulkString(Some("".to_string()))),
        )
        .await
    }

    #[tokio::test]
    async fn test_parse_null_bulk_string() -> Result<()> {
        assert_parse("$-1\r\n", Resp::BulkString(BulkString(None))).await
    }
}
