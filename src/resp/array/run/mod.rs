use std::collections::VecDeque;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::{Array, BulkString, Resp, RespRunResult, RespRunnable, SimpleString};
use crate::storage::Storage;

mod echo;
mod get;
mod ping;
mod set;

impl RespRunnable for Array {
    async fn run(self, storage: &RwLock<Storage>) -> Result<RespRunResult> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = match cmd {
            Resp::SimpleString(SimpleString(s)) | Resp::BulkString(BulkString(Some(s))) => s,
            _ => bail!("invalid command type {}", cmd.to_string()),
        };

        match plain_cmd.to_uppercase().as_str() {
            "PING" => ping::ping(deque),
            "ECHO" => echo::echo(deque),
            "GET" => get::get(deque, storage),
            "SET" => set::set(deque, storage),
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

#[cfg(test)]
mod tests;
