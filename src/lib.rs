//! Simple in-memory <KV> store library, made as part of #100DaysOfCode challenge
//! following the [ping-cap talent plant](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md)
#![deny(missing_docs)]
use std::collections::HashMap;

/*
 * A good structure for documentation (used in stdl) is:
   - [short explanation of what item does]\n
   - [code example showing how to use it]\n
   - [Optional: more expalantions and code examples in case some specific cases are not straightforward]
ref: https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate
guideline: https://rust-lang.github.io/api-guidelines/documentation.html
*/

/// `KvStore` is a simple struct wrapper over a `std::collection::HashMap` to give some abstraction to
/// the <KV> store.
///
/// # Example
///
/// Usage:
/// ```no_run
/// # use kvs::KvStore;
///
/// fn main() {
///   let mut store = KvStore::new();
///   store.set("key".to_owned(), "value".to_owned());
///   let val = match store.get(String::from("key")) {
///       Some(v) => v,
///       None => "no value present".to_owned()  
///   };
///   assert_eq!(val, "value".to_owned());
///   store.remove("key".to_owned());
/// }
///
/// ```
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    ///
    /// ```no_run
    /// # use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// ```
    pub fn new() -> Self {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether the key exists
    ///
    /// # Arguments
    /// * Key: a string represented key for the value to retrieve
    ///
    /// # Example
    /// ```no_run
    /// # use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("key".to_owned(), "value".to_owned());
    /// let val = match store.get(String::from("key")) {
    ///     Some(v) => v,
    ///     None => "no value present".to_owned()  
    /// };
    ///
    /// println!("Value for key {} is {}", "key", val);
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(val) => Some(val.to_owned()),
            _ => None,
        }
    }

    /// Store a value inside the KvStore using a key that can be subsequently used to retrieve the value
    ///
    /// # Arguments
    /// * Key: a string represented key for the value to retrieve
    /// * Value: value to store in string representation
    ///
    /// # Example
    /// ```no_run
    /// # use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("key".to_owned(), "value".to_owned());
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Remove a variable from the KvStore
    ///
    /// # Arguments
    /// * Key: a string represented key for the value to retrieve
    ///
    /// # Example
    /// ```no_run
    /// # use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("key".to_owned(), "value".to_owned());
    /// store.remove("key".to_owned());
    /// ```
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
