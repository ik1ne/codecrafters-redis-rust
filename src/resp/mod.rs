use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use anyhow::{bail, Result};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub use array::Array;
pub use bulk_string::BulkString;
pub use integer::Integer;
pub use simple_string::SimpleString;

use crate::resp::resp_effect::RespEffect;
use crate::storage::Storage;

mod array;
mod bulk_string;
mod integer;
mod simple_string;

mod resp_effect;

trait RespVariant: Display {
    const MAX_BYTES: usize = 1_000_000;
    const PREFIX: char;

    async fn parse_body(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self>
    where
        Self: Sized;
}

trait RespRunnable {
    async fn run(self, storage: &RwLock<Storage>) -> Result<RespEffect>;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Resp {
    SimpleString(SimpleString),
    BulkString(BulkString),
    Array(Array),
    Integer(Integer),
}

impl Display for Resp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resp::SimpleString(s) => write!(f, "{}", s),
            Resp::BulkString(b) => write!(f, "{}", b),
            Resp::Array(a) => write!(f, "{}", a),
            Resp::Integer(i) => write!(f, "{}", i),
        }
    }
}

impl Resp {
    pub async fn parse(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self> {
        let mut prefix = [0; 1];
        let bytes_read = read.read(&mut prefix).await?;

        if bytes_read == 0 {
            // TODO use thiserror to indicate EOF
            bail!("no bytes read, probably EOF");
        }

        macro_rules! parse_body_types {
            [$($tt:tt),*] => {
                $(
                    if <$tt as RespVariant>::PREFIX == prefix[0] as char {
                        return <$tt as RespVariant>::parse_body(read).await.map(Resp::$tt);
                    }
                )*
            };
        }

        parse_body_types![SimpleString, BulkString, Array, Integer];

        bail!("unknown prefix: {:?}", prefix[0] as char);
    }

    pub async fn run(
        self,
        mut write: impl AsyncWrite + Send + Unpin,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<()> {
        async fn run_inner(resp: Resp, storage: &RwLock<Storage>) -> Result<RespEffect> {
            macro_rules! run_types {
                [$($tt:tt),*] => {
                    $(
                        if let Resp::$tt(inner) =
                        resp {
                            return inner.run(storage).await;
                        }
                    )*
                };
            }

            run_types![SimpleString, BulkString, Array];

            bail!("unknown resp type");
        }

        let (run_result, post_run_cmd) = {
            let RespEffect {
                run_result,
                post_run_cmd,
            } = run_inner(self, storage.as_ref()).await?;

            let run_result = run_result.to_string();

            (run_result, post_run_cmd)
        };

        write.write_all(run_result.as_bytes()).await?;

        if let Some(post_run_cmd) = post_run_cmd {
            post_run_cmd.run(write, storage).await?;
        }

        Ok(())
    }

    pub fn plain_string(&self) -> Result<&str> {
        match self {
            Resp::SimpleString(SimpleString(s)) => Ok(s),
            Resp::BulkString(BulkString(s)) => Ok(s.as_deref().unwrap_or("")),
            _ => bail!("not a string"),
        }
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

impl<T: AsyncBufRead + ?Sized> AsyncCrlfReadExt for T {}

#[cfg(test)]
mod tests;
