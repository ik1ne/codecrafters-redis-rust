use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;

use anyhow::{bail, Result};
use tokio::io::AsyncWrite;
use tokio::sync::{RwLock, RwLockReadGuard};

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
        write: impl AsyncWrite + Unpin + Send,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<()> {
        match self {
            PostRespRunCommand::FullResync => {
                if !storage.read().await.is_empty() {
                    bail!("full resync currently only supported for empty storage");
                }

                storage.read().await.encode(write).await
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
