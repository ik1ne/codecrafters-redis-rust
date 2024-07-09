use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::{BulkString, Resp, RespRunResult, RwLockReadGuardedResp};
use crate::storage::Storage;

pub(crate) fn get(mut args: VecDeque<Resp>, storage: &RwLock<Storage>) -> Result<RespRunResult> {
    let key = args.pop_front().context("missing key")?;

    if !args.is_empty() {
        bail!("too many arguments");
    }

    let lock = storage.read().unwrap();
    let Some(value) = lock.get(&key) else {
        return Ok(RespRunResult::Owned(Resp::BulkString(BulkString(None))));
    };

    Ok(RespRunResult::Borrowed(RwLockReadGuardedResp {
        data: NonNull::from(value),
        _guard: lock,
    }))
}
