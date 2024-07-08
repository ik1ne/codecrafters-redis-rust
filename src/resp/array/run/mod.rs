use std::collections::VecDeque;

use anyhow::{bail, Context, Result};

use crate::resp::{Array, BulkString, Resp, RespRunnable, SimpleString};

mod echo;
mod ping;

impl RespRunnable for Array {
    async fn run(self) -> Result<Resp> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = match cmd {
            Resp::SimpleString(SimpleString(s)) | Resp::BulkString(BulkString(Some(s))) => s,
            _ => bail!("invalid command type {}", cmd.to_string()),
        };

        match plain_cmd.to_uppercase().as_str() {
            "PING" => ping::ping(deque),
            "ECHO" => echo::echo(deque),
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

#[cfg(test)]
mod tests;
