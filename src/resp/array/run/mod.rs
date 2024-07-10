use std::collections::VecDeque;
use std::sync::RwLock;

use crate::config::Config;
use crate::resp::{Array, RespRunResult, RespRunnable};
use crate::storage::Storage;
use anyhow::{bail, Context, Result};

mod echo;
mod get;
mod info;
mod ping;
mod set;

impl RespRunnable for Array {
    async fn run<'a>(
        self,
        storage: &'a RwLock<Storage>,
        config: &Config,
    ) -> Result<RespRunResult<'a>> {
        let mut deque = VecDeque::from(self.0);
        let cmd = deque.pop_front().context("empty array")?;

        let plain_cmd = cmd.plain_string().context("invalid command")?;

        match plain_cmd.to_uppercase().as_str() {
            "PING" => ping::ping(deque),
            "ECHO" => echo::echo(deque),
            "GET" => get::get(deque, storage),
            "SET" => set::set(deque, storage),
            "INFO" => info::info(deque, config),
            _ => bail!("unknown command {}", plain_cmd),
        }
    }
}

#[cfg(test)]
mod tests;
