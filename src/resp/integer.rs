use std::fmt::{Display, Formatter};

use anyhow::Result;
use tokio::io::AsyncBufRead;

use crate::resp::{AsyncCrlfReadExt, RespVariant};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Integer(pub i64);

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{}\r\n", self.0)
    }
}

impl RespVariant for Integer {
    const PREFIX: char = ':';

    async fn parse_body(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self> {
        let line = read.read_crlf_line().await?;

        let num = line.parse::<i64>()?;

        Ok(Integer(num))
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::tests::assert_parse;
    use crate::resp::Resp;

    use super::*;

    #[tokio::test]
    async fn test_parse_integer() -> Result<()> {
        assert_parse(":123\r\n", Resp::Integer(Integer(123))).await
    }
}
