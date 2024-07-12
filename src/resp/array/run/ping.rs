use std::collections::VecDeque;

use anyhow::{bail, Result};

use crate::resp::resp_effect::RespRunResult;
use crate::resp::{Resp, RespEffect, SimpleString};

pub async fn ping(mut args: VecDeque<Resp>) -> Result<RespEffect<'static>> {
    let run_result = match args.pop_front() {
        None => RespRunResult::Owned(Resp::SimpleString(SimpleString("PONG".to_string()))),
        Some(message) => {
            if !args.is_empty() {
                bail!("too many arguments");
            }

            RespRunResult::Owned(message)
        }
    };

    Ok(RespEffect {
        run_result,
        post_run_cmd: None,
    })
}
