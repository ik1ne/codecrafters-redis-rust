use std::collections::VecDeque;
use std::sync::RwLock;

use anyhow::{bail, Context, Result};

use crate::resp::{Resp, RespRunResult, SimpleString};
use crate::storage::Storage;

pub(crate) fn set(
    mut args: VecDeque<Resp>,
    storage: &RwLock<Storage>,
) -> Result<RespRunResult<'static>> {
    let key = args.pop_front().context("missing key")?;
    let value = args.pop_front().context("missing value")?;

    if !args.is_empty() {
        bail!("too many arguments");
    }

    let mut storage = storage.write().unwrap();

    storage.set(key, value);

    Ok(RespRunResult::Owned(Resp::SimpleString(SimpleString(
        "OK".to_string(),
    ))))
}
