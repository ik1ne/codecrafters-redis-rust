use std::collections::VecDeque;

use anyhow::bail;

use crate::resp::{Resp, SimpleString};

pub fn ping(mut args: VecDeque<Resp>) -> anyhow::Result<Resp> {
    match args.pop_front() {
        None => Ok(Resp::SimpleString(SimpleString("PONG".to_string()))),
        Some(message) => {
            if !args.is_empty() {
                bail!("too many arguments");
            }

            Ok(message)
        }
    }
}
