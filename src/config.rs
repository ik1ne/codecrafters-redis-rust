use anyhow::{Context, Result};

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub port: u16,
    pub role: Role,
}

impl Config {
    pub(crate) fn parse_args() -> Result<Self> {
        let port = get_arg_value(&mut std::env::args(), "--port")
            .unwrap_or("6379".to_string())
            .parse()?;
        let role = match get_arg_value(&mut std::env::args(), "--replicaof") {
            None => Role::Master,
            Some(master_string) => {
                let mut master_string = master_string.split(' ');
                let host = master_string.next().context("missing master host")?;
                let port = master_string
                    .next()
                    .context("missing master port")?
                    .parse()?;

                Role::Slave {
                    master_host: host.to_string(),
                    master_port: port,
                }
            }
        };

        Ok(Config { port, role })
    }
}

fn get_arg_value(args: &mut std::env::Args, arg_name: &str) -> Option<String> {
    args.find(|arg| arg == arg_name).and_then(|_| args.next())
}

impl Config {
    pub fn replication(&self) -> Vec<String> {
        vec![format!("role:{}", self.role.to_string())]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Role {
    #[default]
    Master,
    Slave {
        master_host: String,
        master_port: u16,
    },
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "master"),
            Role::Slave { .. } => write!(f, "slave"),
        }
    }
}
