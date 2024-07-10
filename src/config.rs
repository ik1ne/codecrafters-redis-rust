use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub role: Role,
}

impl Config {
    pub fn to_vec(&self) -> Vec<String> {
        vec![format!("role:{}", self.role.to_string())]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    #[default]
    Master,
    Slave,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "master"),
            Role::Slave => write!(f, "slave"),
        }
    }
}
