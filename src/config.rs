use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub role: Role,
}

impl Config {
    pub fn parse_parameter(mut parameter: impl Iterator<Item = String>) -> Result<Self> {
        let mut result = HashMap::new();

        while let Some(key) = parameter.next() {
            if !key.starts_with("--") {
                bail!("invalid parameter: {}", key);
            }

            let Some(value) = parameter.next() else {
                bail!("missing value for parameter: {}", key);
            };

            result.insert(key[2..].to_string(), value);
        }

        let port = result
            .get("port")
            .map(|s| s.as_str())
            .unwrap_or("6379")
            .parse()?;

        let role = match result.get("replicaof") {
            None => Role::Master,
            Some(replica_of) => Role::new_slave(replica_of)?,
        };

        Ok(Config { port, role })
    }
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Config {
            port: 6379,
            role: Role::Master,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Master,
    Slave {
        master_host: String,
        master_port: u16,
    },
}

impl Role {
    pub fn new_slave(replica_of: &str) -> Result<Self> {
        let mut replica_of = replica_of.split(' ');
        let host = replica_of
            .next()
            .context("missing master host")?
            .to_string();
        let port = replica_of.next().context("missing master port")?.parse()?;

        Ok(Role::Slave {
            master_host: host,
            master_port: port,
        })
    }
}
