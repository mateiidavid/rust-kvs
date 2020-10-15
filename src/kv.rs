//#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Seek, SeekFrom};
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
    map: HashMap<String, FPos>,
    writer: BufWriter<File>,
    reader: BufReader<File>,
    seek_pos: usize,
}

struct FPos {
    start: usize,
    length: usize,
}

impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    fn new() -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("log.json")?;

        let writer = BufWriter::new(file);

        let file = File::open("log.json")?;
        let reader = BufReader::new(file);
        Ok(KvStore {
            map: HashMap::new(),
            writer,
            reader,
            seek_pos: 0usize,
        })
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether the key exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.replay()?;
        if let Some(fp) = self.map.get(&key) {
            Self::read(&mut self.reader, &mut self.seek_pos, &fp)
        } else {
            Ok(None)
        }
    }

    /// Store a value inside the KvStore using a key that can be subsequently used to retrieve the value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.contains_key(&key);
        let log_cmd = Command::Set { key, value };
        let serialized_cmd = serde_json::to_string(&log_cmd)?;
        writeln!(self.writer, "{}", &serialized_cmd)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Remove a variable from the KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.replay()?;
        if self.map.contains_key(&key) {
            let serialized_cmd = serde_json::to_string(&Command::Rm { key })?;
            writeln!(self.writer, "{}", &serialized_cmd)?;
            self.writer.flush()?;
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
        let mut f = BufReader::new(f);

        let mut line = String::new();
        let mut pos = 0 as usize;
        while let Ok(num) = f.read_line(&mut line) {
            if num == 0 {
                break;
            }
            let cmd: Command = serde_json::from_str(&line)?;
            if let Command::Set { key, .. } = cmd {
                self.map.insert(
                    key,
                    FPos {
                        start: pos,
                        length: num - 1,
                    },
                );
            } else if let Command::Rm { key } = cmd {
                self.map.remove(&key);
            }

            pos += num;
            line.clear();
        }
        Ok(())
    }

    fn read(
        reader: &mut BufReader<File>,
        seek_pos: &mut usize,
        fp: &FPos,
    ) -> Result<Option<String>> {
        let move_by = (fp.start - *seek_pos) as i64;
        let new_pos = reader.seek(SeekFrom::Current(move_by))?;
        *seek_pos = new_pos as usize;
        // how do we check for byte 0?

        let mut buf = vec![0u8; fp.length];
        reader.read_exact(&mut buf)?;
        let buf = String::from_utf8_lossy(&buf);
        let cmd: Command = serde_json::from_str(&buf)?;

        if let Command::Set { key: _, value } = cmd {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
