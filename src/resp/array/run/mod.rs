use std::collections::VecDeque;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::{Array, RespEffect, RespRunnable};
use crate::storage::Storage;

mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;

impl RespRunnable for Array {
    async fn run(self, storage: &RwLock<Storage>) -> Result<RespEffect> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = cmd.plain_string().context("invalid command")?;

        match plain_cmd.to_uppercase().as_str() {
            "ECHO" => echo::echo(deque).await,
            "GET" => get::get(deque, storage).await,
            "INFO" => info::info(deque, storage).await,
            "PING" => ping::ping(deque).await,
            "PSYNC" => psync::psync(deque, storage).await,
            "REPLCONF" => replconf::replconf(deque).await,
            "SET" => set::set(deque, storage).await,
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

#[cfg(test)]
mod tests;
