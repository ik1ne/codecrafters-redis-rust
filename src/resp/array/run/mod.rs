use std::collections::VecDeque;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::{Array, RespRunResult, RespRunnable};
use crate::storage::Storage;

mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;

impl RespRunnable for Array {
    async fn run(self, storage: &RwLock<Storage>) -> Result<RespRunResult> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = cmd.plain_string().context("invalid command")?;

        match plain_cmd.to_uppercase().as_str() {
            "ECHO" => echo::echo(deque),
            "GET" => get::get(deque, storage),
            "INFO" => info::info(deque, storage),
            "PING" => ping::ping(deque),
            "PSYNC" => psync::psync(deque, storage),
            "REPLCONF" => replconf::replconf(deque),
            "SET" => set::set(deque, storage),
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

#[cfg(test)]
mod tests;
