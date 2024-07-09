use std::collections::HashMap;

use crate::resp::Resp;

#[derive(Debug, Default, Clone)]
pub struct Storage {
    data: HashMap<Resp, Resp>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Resp) -> Option<&Resp> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: Resp, value: Resp) {
        self.data.insert(key, value);
    }
}
