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

/// `KvStore` is a simple struct wrapper over a `std::collection::HashMap` to give some abstraction
/// to the <KV> store.
pub struct KvStore {
    map: HashMap<String, FPos>,
    writer: BufPosWriter<File>,
    readers: HashMap<usize, BufPosReader<File>>,
    active_id: usize,
}

struct FPos {
    f_id: usize,
    start: usize,
    length: usize,
}
//TODO: add env var to open @ specific folder
/* Todo: compaction
 * To do compaction, we will do this:
 *  - enforce a limit per file (e.g 2kb or whatever)
 *  - open a new file when the limit is reached
 *    - if we reached the file limit after we flush & close the handle
 *      then merge the files together
 *    - the easiest way to merge is to dump all values in a new file & change the file they point
 *    to
 *    - delete the old files and only have the merged one.\
 *    - mark each file as active, the way Bitcask does.
 *
 * First step: write a file limit
 * Update: we first have to switch to using usize as file id
*/
impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    fn new(
        writer: BufPosWriter<File>,
        readers: HashMap<usize, BufPosReader<File>>,
        active_id: usize,
    ) -> Result<Self> {
        Ok(KvStore {
            map: HashMap::new(),
            writer,
            readers,
            active_id,
        })
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether
    /// the key exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(fp) = self.map.get(&key) {
            if let Some(reader) = self.readers.get_mut(&fp.f_id) {
                Self::read_value(reader, &fp)
            } else {
                Err(KvStoreError::Store(ErrorKind::NotFound))
            }
        } else {
            Ok(None)
        }
    }

    /// Store a value inside the KvStore using a key that can be subsequently used to retrieve
    /// the value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let log_cmd = Command::Set {
            key: key.to_owned(),
            value,
        };
        let cmd = serde_json::to_string(&log_cmd)?;
        let cmd = format!("{}\n", cmd);
        let num = self.writer.write(cmd.as_bytes())?;
        self.map.insert(
            key,
            FPos {
                f_id: self.active_id,
                start: self.writer.pos,
                length: num,
            },
        );
        self.writer.flush()?;
        Ok(())
    }

    /// Remove a variable from the KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.map.contains_key(&key) {
            let cmd = serde_json::to_string(&Command::Rm {
                key: key.to_owned(),
            })?;
            let cmd = format!("{}\n", cmd);
            let num = self.writer.write(cmd.as_bytes())?;
            self.map.insert(
                key,
                FPos {
                    f_id: self.active_id,
                    start: self.writer.pos,
                    length: num,
                },
            );
            self.writer.flush()?;
            Ok(())
        } else {
            Err(KvStoreError::Store(ErrorKind::NotFound))
        }
    }

    ///
    //TODO: Currently, we want to switch from keeping string values to just using file ids
    // for now, it won't be smart NOT to deal with file name as string, lots of conversions to do
    // what we should do (maybe) is do a conversion, keep array of just file numbers and figure out
    // which one is max to denote active file
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;
        env::set_current_dir(&path)?;
        let files = std::fs::read_dir(&path)?
            .filter_map(std::io::Result::ok)
            .filter_map(|e| match e.file_name().to_str() {
                Some(v) => {
                    if v.contains("log") {
                        Some(v.to_owned())
                    } else {
                        None
                    }
                }

                _ => None,
            })
            .collect::<Vec<String>>();

        let mut readers: HashMap<usize, BufPosReader<File>> = HashMap::new();
        for f_name in files.iter() {
            let file = File::open(&f_name)?;
            let reader = BufPosReader::new(file)?;
            //TODO: Remove unwraps, looks ugly af
            let f_name = f_name
                .find('-')
                .map(|i| Some(f_name[..i].to_owned()))
                .map(|name| name.unwrap().parse::<usize>().unwrap())
                .unwrap();

            readers.insert(f_name, reader);
        }

        let active_id: usize;
        let active_file: File;
        if let Some(num) = readers.keys().max() {
            active_id = *num;
            let file_name = format!("{}-log.json", num);
            active_file = std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .open(file_name)?;
        } else {
            active_id = 0;
            active_file = std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open("0-log.json")?;

            let f = File::open("0-log.json")?;
            let f = BufPosReader::new(f)?;
            readers.insert(0, f);
        }
        let writer = BufPosWriter::new(active_file)?;

        let mut store = KvStore::new(writer, readers, active_id)?;
        store.replay(files)?;
        Ok(store)
    }

    pub fn replay(&mut self, files: Vec<String>) -> Result<()> {
        for f_name in files {
            let f = std::fs::File::open(&f_name)?;
            let mut f = BufReader::new(f);

            let mut line = String::new();
            let mut pos = 0 as usize;
            while let Ok(num) = f.read_line(&mut line) {
                if num == 0 {
                    break;
                }
                let cmd: Command = serde_json::from_str(&line)?;
                let f_id = f_name
                    .find('-')
                    .map(|i| Some(f_name[..i].to_owned()))
                    .map(|name| name.unwrap().parse::<usize>().unwrap())
                    .unwrap();
                if let Command::Set { key, .. } = cmd {
                    self.map.insert(
                        key,
                        FPos {
                            f_id,
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
        }
        Ok(())
    }

    fn read_value(reader: &mut BufPosReader<File>, fp: &FPos) -> Result<Option<String>> {
        let move_by = (fp.start - reader.pos) as i64;
        let mut buf = vec![0u8; fp.length];
        reader.seek(SeekFrom::Current(move_by))?;

        reader.read(&mut buf)?;
        let buf = String::from_utf8_lossy(&buf);
        let cmd: Command = serde_json::from_str(&buf)?;

        if let Command::Set { key: _, value } = cmd {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct BufPosWriter<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: usize,
}

impl<W: Write + Seek> BufPosWriter<W> {
    fn new(mut f: W) -> Result<Self> {
        let pos = f.seek(SeekFrom::Current(0))? as usize;
        Ok(BufPosWriter {
            writer: BufWriter::new(f),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufPosWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes = self.writer.write(buf)?;
        self.pos += bytes;
        Ok(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

struct BufPosReader<R: Read + Seek> {
    reader: BufReader<R>,
    pos: usize,
}

impl<R: Read + Seek> BufPosReader<R> {
    fn new(mut f: R) -> Result<Self> {
        // TODO: do we need this line?
        let pos = f.seek(SeekFrom::Start(0))? as usize;
        Ok(BufPosReader {
            reader: BufReader::new(f),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufPosReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = self.reader.read(buf)?;
        self.pos += bytes;
        Ok(bytes)
    }
}

impl<R: Read + Seek> Seek for BufPosReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let bytes = self.reader.seek(pos)?;
        self.pos = bytes as usize;
        Ok(bytes)
    }
}
