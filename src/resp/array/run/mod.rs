use std::collections::VecDeque;

use anyhow::{bail, Context, Result};

use crate::resp::{Array, BulkString, Resp, RespRunnable, SimpleString};

impl RespRunnable for Array {
    async fn run(self) -> Result<Resp> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = match cmd {
            Resp::SimpleString(SimpleString(s)) | Resp::BulkString(BulkString(Some(s))) => s,
            _ => bail!("invalid command type {}", cmd.to_string()),
        };

        match plain_cmd.to_uppercase().as_str() {
            "PING" => pong(deque),
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

fn pong(mut args: VecDeque<Resp>) -> Result<Resp> {
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

#[cfg(test)]
mod tests;
