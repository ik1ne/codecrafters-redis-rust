use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::resp_effect::{RespEffect, RespRunResult, RwLockReadGuardedResp};
use crate::resp::{BulkString, Resp};
use crate::storage::Storage;

pub async fn get(mut args: VecDeque<Resp>, storage: &RwLock<Storage>) -> Result<RespEffect> {
    let key = args.pop_front().context("missing key")?;

    if !args.is_empty() {
        bail!("too many arguments");
    }

    let lock = storage.read().unwrap();
    let Some(value) = lock.get(&key) else {
        return Ok(RespEffect {
            run_result: RespRunResult::Owned(Resp::BulkString(BulkString(None))),
            post_run_cmd: None,
        });
    };

    Ok(RespEffect {
        run_result: RespRunResult::Borrowed(RwLockReadGuardedResp {
            data: NonNull::from(value),
            _guard: lock,
        }),
        post_run_cmd: None,
    })
}
