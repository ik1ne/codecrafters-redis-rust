use anyhow::Result;
use tokio::io::AsyncBufRead;

use crate::resp::{AsyncCrlfReadExt, RespParsable};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SimpleString(pub String);

impl RespParsable for SimpleString {
    const PREFIX: char = '+';

    async fn parse_body(mut read: impl AsyncBufRead + Unpin + Send) -> Result<Self> {
        let line = read.read_crlf_line().await?;

        Ok(SimpleString(line))
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::tests::assert_parse;
    use crate::resp::Resp;

    use super::*;

    #[tokio::test]
    async fn test_parse_simple_string() -> Result<()> {
        assert_parse(
            "+OK\r\n",
            Resp::SimpleString(SimpleString("OK".to_string())),
        )
        .await
    }

    #[tokio::test]
    async fn test_parse_empty_simple_string() -> Result<()> {
        assert_parse("+\r\n", Resp::SimpleString(SimpleString("".to_string()))).await
    }
}
