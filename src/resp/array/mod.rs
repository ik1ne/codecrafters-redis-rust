use std::fmt::{Display, Formatter};
use std::future::Future;

use anyhow::Result;
use tokio::io::AsyncBufRead;

use crate::resp::{AsyncCrlfReadExt, Resp, RespVariant};

mod run;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Array(Vec<Resp>);

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{}\r\n", self.0.len())?;
        for element in &self.0 {
            write!(f, "{}", element)?;
        }
        Ok(())
    }
}

impl RespVariant for Array {
    const PREFIX: char = '*';

    fn parse_body(
        read: &mut (impl AsyncBufRead + Unpin + Send),
    ) -> impl Future<Output = Result<Self>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let num_elements = read.read_crlf_line().await?.parse::<usize>()?;

            let mut elements = Vec::with_capacity(num_elements);

            for _ in 0..num_elements {
                let element = Resp::parse(read).await?;
                elements.push(element);
            }

            Ok(Array(elements))
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::resp::bulk_string::BulkString;
    use crate::resp::tests::assert_parse;
    use crate::resp::Resp;

    use super::*;

    #[tokio::test]
    async fn test_parse_single_element_array() -> Result<()> {
        assert_parse(
            "*1\r\n$5\r\nhello\r\n",
            Resp::Array(Array(vec![Resp::BulkString(BulkString(Some(
                "hello".to_string(),
            )))])),
        )
        .await
    }

    #[tokio::test]
    async fn test_parse_multiple_elements_array() -> Result<()> {
        assert_parse(
            "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
            Resp::Array(Array(vec![
                Resp::BulkString(BulkString(Some("hello".to_string()))),
                Resp::BulkString(BulkString(Some("world".to_string()))),
            ])),
        )
        .await
    }

    #[tokio::test]
    async fn test_parse_empty_array() -> Result<()> {
        assert_parse("*0\r\n", Resp::Array(Array(vec![]))).await
    }
}
