use std::collections::VecDeque;

use anyhow::{anyhow, bail, Result};
use tokio::sync::RwLock;

use crate::resp::resp_effect::{RespEffect, RespRunResult};
use crate::resp::{BulkString, Resp};
use crate::storage::Storage;

pub async fn info(
    mut args: VecDeque<Resp>,
    storage: &RwLock<Storage>,
) -> Result<RespEffect<'static>> {
    let first_arg = args
        .pop_front()
        .ok_or_else(|| anyhow!("missing info target"))?;

    let info_target = first_arg.plain_string()?;

    let s = match info_target.to_lowercase().as_str() {
        "replication" => storage.read().await.replication.info(),
        _ => bail!("unsupported info target: {}", info_target),
    };

    Ok(RespEffect {
        run_result: RespRunResult::Owned(Resp::BulkString(BulkString(Some(s)))),
        post_run_cmd: None,
    })
}
