use std::collections::VecDeque;
use std::sync::RwLock;

use anyhow::{ensure, Context, Result};

use crate::resp::{Resp, RespRunResult, SimpleString};
use crate::storage::Storage;

pub(crate) fn psync(mut args: VecDeque<Resp>, storage: &RwLock<Storage>) -> Result<RespRunResult> {
    let replid = args.pop_front().context("missing replid")?;
    let replid = replid.plain_string()?;

    ensure!(replid == "?", "only replid value of ? is supported for now");

    let offset = args.pop_front().context("missing offset")?;
    let offset = offset.plain_string()?.parse::<i128>()?;

    ensure!(offset == -1, "only offset value of -1 is supported for now");

    let psync_info = storage
        .read()
        .unwrap()
        .replication
        .info_psync()
        .context("no replication info")?;

    let reply = Resp::SimpleString(SimpleString(format!("FULLRESYNC {}", psync_info)));

    Ok(RespRunResult::Owned(reply))
}
