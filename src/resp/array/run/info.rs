use std::collections::VecDeque;

use anyhow::Result;

use crate::config::Config;
use crate::resp::{BulkString, Resp, RespRunResult};

pub fn info(_args: VecDeque<Resp>, config: &Config) -> Result<RespRunResult<'static>> {
    Ok(RespRunResult::Owned(Resp::BulkString(BulkString(Some(
        config.to_vec().join("\n"),
    )))))
}
