use std::collections::VecDeque;

use anyhow::{bail, Context, Result};

use crate::resp::resp_effect::{RespEffect, RespRunResult};
use crate::resp::Resp;

pub async fn echo(mut args: VecDeque<Resp>) -> Result<RespEffect<'static>> {
    let message = args.pop_front().context("missing message")?;

    if !args.is_empty() {
        bail!("too many arguments");
    }

    Ok(RespEffect {
        run_result: RespRunResult::Owned(message),
        post_run_cmd: None,
    })
}
