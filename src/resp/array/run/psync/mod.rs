use std::collections::VecDeque;

use anyhow::{ensure, Context, Result};
use tokio::sync::RwLock;

use crate::resp::resp_effect::{PostRespRunCommand, RespEffect, RespRunResult};
use crate::resp::{Resp, SimpleString};
use crate::storage::Storage;

pub async fn psync(mut args: VecDeque<Resp>, storage: &RwLock<Storage>) -> Result<RespEffect> {
    let replid = args.pop_front().context("missing replid")?;
    let replid = replid.plain_string()?;

    ensure!(replid == "?", "only replid value of ? is supported for now");

    let offset = args.pop_front().context("missing offset")?;
    let offset = offset.plain_string()?.parse::<i128>()?;

    ensure!(offset == -1, "only offset value of -1 is supported for now");

    let psync_info = storage
        .read()
        .await
        .replication
        .info_psync()
        .context("no replication info")?;

    let reply = Resp::SimpleString(SimpleString(format!("FULLRESYNC {}", psync_info)));

    Ok(RespEffect {
        run_result: RespRunResult::Owned(reply),
        post_run_cmd: Some(PostRespRunCommand::FullResync),
    })
}
