//! Simple in-memory <KV> store library, made as part of #100DaysOfCode challenge
//! following the [ping-cap talent plant](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md)
//#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fmt, io, io::Write, path::PathBuf};
/*
 * A good structure for documentation (used in stdl) is:
   - [short explanation of what item does]\n
   - [code example showing how to use it]\n
   - [Optional: more expalantions and code examples in case some specific cases are not straightforward]
ref: https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate
guideline: https://rust-lang.github.io/api-guidelines/documentation.html
*/

#[derive(Debug)]
/// A union of all possible errors in our lib
pub enum KvStoreError {
    // Errors from ext libs
    Io(io::Error),
    Serde(serde_json::Error),
    // Errors from this lib
    Regular(ErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, PartialEq, Ord)]
pub enum ErrorKind {
    NotFound,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match self {
            ErrorKind::NotFound => "not found",
            _ => "something else",
        }
    }
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvStoreError::Io(err) => err.fmt(f),
            KvStoreError::Serde(err) => err.fmt(f),
            KvStoreError::Regular(err) => write!(f, "KvStore error occurred {:?}", err),
        }
    }
}

// Enable ? op
impl From<io::Error> for KvStoreError {
    fn from(err: io::Error) -> KvStoreError {
        KvStoreError::Io(err)
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(err: serde_json::Error) -> KvStoreError {
        KvStoreError::Serde(err)
    }
}
///
pub type Result<T> = std::result::Result<T, KvStoreError>;

// For now, will pick JSON but as I benchmark I will be thinking
// of moving to MessagePack
#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
enum Command {
    Set { key: String, value: String },
    Rm { key: String },
}

/// `KvStore` is a simple struct wrapper over a `std::collection::HashMap` to give some abstraction to
/// the <KV> store.
pub struct KvStore {
    map: HashMap<String, String>,
    file_buf: io::BufWriter<std::fs::File>,
}

impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    fn new() -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("log.json")?;

        let file_buf = std::io::BufWriter::new(file);

        Ok(KvStore {
            map: HashMap::new(),
            file_buf,
        })
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether the key exists
    pub fn get(&self, key: String) -> Result<Option<String>> {
        unimplemented!("Get used to be implemented but now it's not. Oops")
    }

    /// Store a value inside the KvStore using a key that can be subsequently used to retrieve the value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let log_cmd = Command::Set { key, value };
        let serialized_cmd = serde_json::to_string(&log_cmd)?;
        writeln!(self.file_buf, "{},", &serialized_cmd)?;
        self.file_buf.flush()?;
        Ok(())
    }

    /// Remove a variable from the KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.map.remove(&key);
        Ok(())
    }

    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;
        env::set_current_dir(&path)?;
        let store = KvStore::new()?;
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
