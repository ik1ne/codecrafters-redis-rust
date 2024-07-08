use std::collections::VecDeque;

use anyhow::bail;

use crate::resp::Resp;

pub fn echo(mut args: VecDeque<Resp>) -> anyhow::Result<Resp> {
    match args.pop_front() {
        None => bail!("missing argument"),
        Some(message) => {
            if !args.is_empty() {
                bail!("too many arguments");
            }

            Ok(message)
        }
    }
}
