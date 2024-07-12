use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Result};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::config::{Config, Role};
use crate::resp::Resp;
use crate::utils::unhex;

const RANDOM_REPLID: &str = "random_replid";

fn random_repl_id() -> String {
    RANDOM_REPLID.to_string()
}

#[derive(Debug, Default, Clone)]
pub struct Storage {
    data: HashMap<Resp, (Resp, Option<SystemTime>)>,
    pub replication: Replication,
}

#[derive(Debug, Clone)]
pub enum Replication {
    Master {
        replid: String,
        offset: u64,
    },
    Slave {
        master_host: String,
        master_port: u16,
    },
}

impl Default for Replication {
    fn default() -> Self {
        Replication::Master {
            replid: random_repl_id(),
            offset: 0,
        }
    }
}

impl Replication {
    pub fn info(&self) -> String {
        match self {
            Replication::Master { replid, offset } => [
                "role:master".to_string(),
                format!("master_replid:{}", replid),
                format!("master_repl_offset:{}", offset),
            ]
            .join("\n"),
            Replication::Slave {
                master_host,
                master_port,
            } => [
                "role:slave".to_string(),
                format!("master_host:{}", master_host),
                format!("master_port:{}", master_port),
            ]
            .join("\n"),
        }
    }

    pub fn info_psync(&self) -> Option<String> {
        match self {
            Replication::Master { replid, offset } => Some(format!("{} {}", replid, offset)),
            Replication::Slave { .. } => None,
        }
    }
}

impl Storage {
    pub fn new(config: &Config) -> Self {
        let data = HashMap::new();
        let replication = match config.role {
            Role::Master => Replication::default(),
            Role::Slave {
                ref master_host,
                master_port,
            } => Replication::Slave {
                master_host: master_host.clone(),
                master_port,
            },
        };

        Storage { data, replication }
    }
}

impl Storage {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, key: &Resp) -> Option<&Resp> {
        match self.data.get(key) {
            None => None,
            Some((resp, expiry)) => {
                if let Some(expiry) = *expiry {
                    if expiry < SystemTime::now() {
                        None
                    } else {
                        Some(resp)
                    }
                } else {
                    Some(resp)
                }
            }
        }
    }

    pub fn set(&mut self, key: Resp, value: Resp, expiry: Option<Duration>) {
        self.data
            .insert(key, (value, expiry.map(|d| SystemTime::now() + d)));
    }

    pub async fn encode(&self, mut write: impl AsyncWrite + Send + Unpin) -> Result<()> {
        if !self.is_empty() {
            bail!("full resync currently only supported for empty storage");
        }

        const EMPTY_RDB_FILE_HEX:&str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";

        let bytes = unhex(EMPTY_RDB_FILE_HEX)?;

        write.write_all("$".as_bytes()).await?;
        write
            .write_all(format!("{}\r\n", bytes.len()).as_bytes())
            .await?;
        write.write_all(&bytes).await?;

        Ok(())
    }
}
