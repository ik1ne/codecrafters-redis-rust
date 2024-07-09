use std::collections::VecDeque;

use anyhow::{bail, Context, Result};

use crate::resp::{Resp, RespRunResult};

pub fn echo(mut args: VecDeque<Resp>) -> Result<RespRunResult<'static>> {
    let message = args.pop_front().context("missing message")?;

    if !args.is_empty() {
        bail!("too many arguments");
    }

    Ok(RespRunResult::Owned(message))
}
