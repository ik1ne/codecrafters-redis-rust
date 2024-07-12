use std::collections::VecDeque;

use anyhow::Result;

use crate::resp::{Resp, RespRunResult, SimpleString};

pub(crate) fn replconf(_args: VecDeque<Resp>) -> Result<RespRunResult<'static>> {
    Ok(RespRunResult::Owned(Resp::SimpleString(SimpleString(
        "OK".to_string(),
    ))))
}
