use std::collections::VecDeque;

use anyhow::{bail, Result};

use crate::resp::{Resp, RespRunResult, SimpleString};

pub fn ping<'a>(mut args: VecDeque<Resp>) -> Result<RespRunResult<'a>> {
    match args.pop_front() {
        None => Ok(RespRunResult::Owned(Resp::SimpleString(SimpleString(
            "PONG".to_string(),
        )))),
        Some(message) => {
            if !args.is_empty() {
                bail!("too many arguments");
            }

            Ok(RespRunResult::Owned(message))
        }
    }
}
