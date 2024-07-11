use anyhow::{anyhow, bail, Result};
use std::collections::VecDeque;
use std::sync::RwLock;

use crate::resp::{BulkString, Resp, RespRunResult};
use crate::storage::Storage;

pub fn info(mut args: VecDeque<Resp>, storage: &RwLock<Storage>) -> Result<RespRunResult<'static>> {
    let first_arg = args
        .pop_front()
        .ok_or_else(|| anyhow!("missing info target"))?;

    let info_target = first_arg.plain_string()?;

    let s = match info_target.to_lowercase().as_str() {
        "replication" => storage.read().unwrap().replication.info(),
        _ => bail!("unsupported info target: {}", info_target),
    };

    Ok(RespRunResult::Owned(Resp::BulkString(BulkString(Some(s)))))
}
