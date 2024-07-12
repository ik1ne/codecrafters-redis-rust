use std::collections::VecDeque;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::sync::RwLock;

use crate::resp::integer::Integer;
use crate::resp::resp_effect::{RespEffect, RespRunResult};
use crate::resp::{BulkString, Resp, SimpleString};
use crate::storage::Storage;

pub async fn set(
    mut args: VecDeque<Resp>,
    storage: &RwLock<Storage>,
) -> Result<RespEffect<'static>> {
    let key = args.pop_front().context("missing key")?;
    let value = args.pop_front().context("missing value")?;

    let expiry = if let Some(px) = args.pop_front() {
        if px.plain_string()?.to_uppercase() != "PX" {
            bail!("unknown argument {}", px.to_string());
        }

        let expiry_i64 = match args.pop_front().context("missing px value")? {
            // NOTE: Codecrafters send the px value as a bulk string instead of Integer
            Resp::SimpleString(SimpleString(s)) | Resp::BulkString(BulkString(Some(s))) => {
                s.parse::<i64>()?
            }
            Resp::Integer(Integer(i)) => i,
            _ => bail!("invalid px value"),
        };

        if expiry_i64 <= 0 {
            bail!("invalid px value {}", expiry_i64);
        }

        Some(Duration::from_millis(expiry_i64 as u64))
    } else {
        None
    };

    if !args.is_empty() {
        bail!("too many arguments");
    }

    let mut storage = storage.write().await;

    storage.set(key, value, expiry);

    Ok(RespEffect {
        run_result: RespRunResult::Owned(Resp::SimpleString(SimpleString("OK".to_string()))),
        post_run_cmd: None,
    })
}
