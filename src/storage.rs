use crate::resp::Resp;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Default, Clone)]
pub struct Storage {
    data: HashMap<Resp, (Resp, Option<SystemTime>)>,
}

impl Storage {
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
}
