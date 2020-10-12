//#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, BufReader, BufWriter};
use std::{collections::HashMap, env, path::PathBuf};

use crate::{ErrorKind, KvStoreError, Result};
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
    write_buf: BufWriter<std::fs::File>,
}

impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    fn new() -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("log.json")?;

        let mut write_buf = BufWriter::new(file);

        Ok(KvStore {
            map: HashMap::new(),
            write_buf,
        })
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether the key exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.replay()?;
        match self.map.get(&key) {
            Some(v) => Ok(Some(v.to_owned())),
            None => Ok(None),
        }
    }

    /// Store a value inside the KvStore using a key that can be subsequently used to retrieve the value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.insert(key.clone(), value.clone());
        self.map.contains_key(&key);
        let log_cmd = Command::Set { key, value };
        let serialized_cmd = serde_json::to_string(&log_cmd)?;
        writeln!(self.write_buf, "{}", &serialized_cmd)?;
        self.write_buf.flush()?;
        Ok(())
    }

    /// Remove a variable from the KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.replay()?;
        if self.map.contains_key(&key) {
            let serialized_cmd = serde_json::to_string(&Command::Rm { key })?;
            writeln!(self.write_buf, "{}", &serialized_cmd)?;
            self.write_buf.flush()?;
            Ok(())
        } else {
            Err(KvStoreError::Store(ErrorKind::NotFound))
        }
    }

    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;
        env::set_current_dir(&path)?;
        let store = KvStore::new()?;
        Ok(store)
    }

    pub fn replay(&mut self) -> Result<()> {
        let f = std::fs::File::open("log.json")?;
        let f = BufReader::new(f);

        for line in f.lines() {
            let line = line.expect("could not read line");
            let cmd: Command = serde_json::from_str(&line)?;
            if let Command::Set { key, value } = cmd {
                self.map.insert(key, value);
            } else if let Command::Rm { key } = cmd {
                self.map.remove(&key);
            }
        }
        Ok(())
    }
}
