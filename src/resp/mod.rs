use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use anyhow::{bail, Result};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub use array::Array;
pub use bulk_string::BulkString;
pub use simple_string::SimpleString;

use crate::storage::Storage;

mod array;
mod bulk_string;
mod simple_string;

trait RespVariant: Display {
    const MAX_BYTES: usize = 1_000_000;
    const PREFIX: char;

    async fn parse_body(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self>
    where
        Self: Sized;
}

trait RespRunnable {
    async fn run(self, storage: &RwLock<Storage>) -> Result<RespRunResult>;
}

enum RespRunResult<'a> {
    Owned(Resp),
    Borrowed(RwLockReadGuardedResp<'a>),
}

impl<'a> Deref for RespRunResult<'a> {
    type Target = Resp;

    fn deref(&self) -> &Self::Target {
        match self {
            RespRunResult::Owned(resp) => resp,
            RespRunResult::Borrowed(resp) => resp,
        }
    }
}

struct RwLockReadGuardedResp<'a> {
    data: NonNull<Resp>,
    _guard: RwLockReadGuard<'a, Storage>,
}

impl<'a> Deref for RwLockReadGuardedResp<'a> {
    type Target = Resp;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Resp {
    SimpleString(SimpleString),
    BulkString(BulkString),
    Array(Array),
}

impl Display for Resp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resp::SimpleString(s) => write!(f, "{}", s),
            Resp::BulkString(b) => write!(f, "{}", b),
            Resp::Array(a) => write!(f, "{}", a),
        }
    }
}

impl Resp {
    pub async fn parse(read: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self> {
        let mut prefix = [0; 1];
        let bytes_read = read.read(&mut prefix).await?;

        if bytes_read == 0 {
            bail!("no prefix read");
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

        parse_body_types![SimpleString, BulkString, Array];

        bail!("unknown prefix: {:?}", prefix[0] as char);
    }

    pub async fn run(
        self,
        mut write: impl AsyncWrite + Unpin,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<()> {
        async fn run_inner(resp: Resp, storage: &RwLock<Storage>) -> Result<RespRunResult> {
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

        let string = {
            let resp = run_inner(self, storage.as_ref()).await?;
            resp.to_string()
        };

        write.write_all(string.as_bytes()).await?;

        Ok(())
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
