use std::collections::VecDeque;

use anyhow::{anyhow, bail, Result};

use crate::config::Config;
use crate::resp::{BulkString, Resp, RespRunResult};

pub fn info(mut args: VecDeque<Resp>, config: &Config) -> Result<RespRunResult<'static>> {
    let first_arg = args
        .pop_front()
        .ok_or_else(|| anyhow!("missing info target"))?;

    let info_target = first_arg.plain_string()?;

    let s = match info_target.to_lowercase().as_str() {
        "replication" => config.replication().join("\n"),
        _ => bail!("unsupported info target: {}", info_target),
    };

    Ok(RespRunResult::Owned(Resp::BulkString(BulkString(Some(s)))))
}
