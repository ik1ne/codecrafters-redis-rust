use anyhow::{bail, Result};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite};

pub use array::Array;
pub use bulk_string::BulkString;
pub use simple_string::SimpleString;

mod array;
mod bulk_string;
mod simple_string;

pub const MAX_BYTES: usize = 1_000_000;

trait RespParsable {
    const MAX_BYTES: usize = 1_000_000;
    const PREFIX: char;

    async fn parse_body(read: impl AsyncBufRead + Unpin) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Resp {
    SimpleString(SimpleString),
    BulkString(BulkString),
    Array(Array),
}

impl Resp {
    pub async fn parse(mut read: impl AsyncBufRead + Unpin) -> Result<Self> {
        let mut prefix = [0; 1];
        let bytes_read = read.read(&mut prefix).await?;

        if bytes_read == 0 {
            bail!("no prefix read");
        }

        macro_rules! parse_body_types {
            [$($tt:tt),*] => {
                $(
                    if <$tt as RespParsable>::PREFIX == prefix[0] as char {
                        return <$tt as RespParsable>::parse_body(read).await.map(Resp::$tt);
                    }
                )*
            };
        }

        parse_body_types![SimpleString, BulkString, Array];

        bail!("unknown prefix: {:?}", prefix[0] as char);
    }

    pub async fn run(self, write: impl AsyncWrite + Unpin) -> Result<()> {
        todo!()
    }
}

trait AsyncCrlfReadExt: AsyncBufRead {
    async fn read_crlf_line(&mut self) -> Result<String>
    where
        Self: Unpin,
    {
        let mut line = String::new();
        while !line.ends_with("\r\n") {
            let bytes_read = self.read_line(&mut line).await?;
            if bytes_read == 0 {
                bail!("no bytes read");
            }
        }

        line.pop();
        line.pop();

        Ok(line)
    }

    async fn read_bytes(&mut self, num_bytes: u64) -> Result<Vec<u8>>
    where
        Self: Unpin,
    {
        let mut buf = vec![];
        self.take(num_bytes).read_to_end(&mut buf).await?;

        Ok(buf)
    }
}

impl<T: AsyncBufReadExt + ?Sized> AsyncCrlfReadExt for T {}

#[cfg(test)]
mod tests;
