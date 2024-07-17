use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use anyhow::{bail, Result};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::resp::Resp;
use crate::storage::Storage;

#[derive(Debug)]
pub struct RespEffect<'a> {
    pub run_result: RespRunResult<'a>,
    pub post_run_cmd: Option<PostRespRunCommand>,
}

#[derive(Debug)]
pub enum RespRunResult<'a> {
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

#[derive(Debug)]
pub struct RwLockReadGuardedResp<'a> {
    pub data: NonNull<Resp>,
    pub _guard: RwLockReadGuard<'a, Storage>,
}

#[derive(Debug, Clone, Copy)]
pub enum PostRespRunCommand {
    FullResync,
}

impl PostRespRunCommand {
    pub async fn run(
        &self,
        mut write: impl AsyncWrite + Unpin + Send,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<()> {
        match self {
            PostRespRunCommand::FullResync => {
                if !storage.read().unwrap().is_empty() {
                    bail!("full resync currently only supported for empty storage");
                }

                let encoded = storage.read().unwrap().encode()?;

                write.write_all(&encoded).await?;

                Ok(())
            }
        }
    }
}

impl<'a> Deref for RwLockReadGuardedResp<'a> {
    type Target = Resp;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}
