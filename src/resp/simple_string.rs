use anyhow::{bail, Result};
use std::fmt::{Display, Formatter};
use std::sync::RwLock;
use tokio::io::AsyncBufRead;

use crate::resp::{AsyncCrlfReadExt, Resp, RespRunResult, RespRunnable, RespVariant};
use crate::storage::Storage;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SimpleString(pub String);

impl Display for SimpleString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}\r\n", self.0)
    }
}

impl RespVariant for SimpleString {
    const PREFIX: char = '+';

    async fn parse_body(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self> {
        let line = read.read_crlf_line().await?;

        Ok(SimpleString(line))
    }
}

impl RespRunnable for SimpleString {
    async fn run(self, _storage: &RwLock<Storage>) -> Result<RespRunResult> {
        Ok(RespRunResult::Owned(run_string(self.0)?))
    }
}

pub(super) fn run_string(s: String) -> Result<Resp> {
    match s.as_str() {
        "PING" => Ok(Resp::SimpleString(SimpleString("PONG".to_string()))),
        _ => bail!("unknown command"),
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
