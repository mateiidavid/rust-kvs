use std::collections::HashMap;

pub struct KvStore {
    map: HashMap<String, String>, 
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {map: HashMap::new()}
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(val) => Some(val.to_owned()),
            _ => None
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
