use std::collections::VecDeque;

use anyhow::Result;

use crate::resp::resp_effect::{RespEffect, RespRunResult};
use crate::resp::{Resp, SimpleString};

pub async fn replconf(_args: VecDeque<Resp>) -> Result<RespEffect<'static>> {
    Ok(RespEffect {
        run_result: RespRunResult::Owned(Resp::SimpleString(SimpleString("OK".to_string()))),
        post_run_cmd: None,
    })
}
