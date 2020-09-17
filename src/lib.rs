pub struct KvStore {}

impl KvStore {
    pub fn new() -> Self {
        KvStore {}
    }

    pub fn get(&self, key: String) -> Option<String> {
        panic!()
    }

    pub fn set(&self, key: String, value: String) {
        panic!()
    }

    pub fn remove(&self, key: String) {
        panic!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
