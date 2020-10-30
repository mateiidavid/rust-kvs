//#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Seek, SeekFrom};
use std::{collections::HashMap, path::Path, path::PathBuf};

use crate::{ErrorKind, KvStoreError, Result};
// For now, will pick JSON but as I benchmark I will be thinking
// of moving to MessagePack
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
enum Command {
    Set { key: String, value: String },
    Rm { key: String },
}

/// `KvStore` is a simple struct wrapper over a `std::collection::HashMap` to give some abstraction
/// to the <KV> store.
pub struct KvStore {
    idx: HashMap<String, CmdPos>,
    writer: BufPosWriter<File>,
    readers: HashMap<usize, BufPosReader<File>>,
    active_id: usize,
    total_sz: usize,
    path: PathBuf, // Credit to pingcap guide
}

#[derive(Debug)]
pub struct CmdPos {
    f_id: usize,
    pos: usize,
    sz: usize,
}

const MAX_STORE_SZ: usize = 2048;
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
 * Step now: implement total_sz to keep track of size of current active file
*/
impl KvStore {
    /// Create a new instance of KvStore by in turn creating a HashMap
    fn new(
        writer: BufPosWriter<File>,
        readers: HashMap<usize, BufPosReader<File>>,
        idx: HashMap<String, CmdPos>,
        active_id: usize,
        total_sz: usize,
        path: PathBuf,
    ) -> Result<Self> {
        Ok(KvStore {
            idx,
            writer,
            readers,
            active_id,
            total_sz,
            path,
        })
    }

    /// Retrieve a variable from the KvStore and return as an Option<String> depending on whether
    /// the key exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(p) = self.idx.get(&key) {
            let reader = self
                .readers
                .get_mut(&p.f_id)
                .expect("could not get reader associated with key");
            let mut buf = vec![0u8; p.sz];
            reader.seek(SeekFrom::Start(p.pos as u64))?;
            reader.read(&mut buf)?;
            let cmd: Command = serde_json::from_slice(&buf)?;
            if let Command::Set { key: _, value } = cmd {
                Ok(Some(value))
            } else {
                Err(KvStoreError::Store(ErrorKind::UnsupportedCommand))
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
        let pos = self.writer.pos;
        let sz = self.writer.write(cmd.as_bytes())?;
        let pos = CmdPos {
            f_id: self.active_id,
            pos,
            sz,
        };
        self.idx.insert(key, pos);
        self.writer.flush()?;
        self.total_sz += sz;

        if self.total_sz >= MAX_STORE_SZ {
            self.compact()
        } else {
            Ok(())
        }
    }

    /// Remove a variable from the KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.idx.contains_key(&key) {
            let cmd = serde_json::to_string(&Command::Rm {
                key: key.to_owned(),
            })?;
            let sz = self.writer.write(cmd.as_bytes())?;
            self.idx.remove(&key);
            self.writer.flush()?;
            self.total_sz += sz;

            if self.total_sz >= MAX_STORE_SZ {
                self.compact()
            } else {
                Ok(())
            }
        } else {
            Err(KvStoreError::Store(ErrorKind::NotFound))
        }
    }

    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;
        let mut files = std::fs::read_dir(&path)?
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

        files.sort();
        let mut total_sz = 0usize;
        let mut readers: HashMap<usize, BufPosReader<File>> = HashMap::new();
        let mut idx: HashMap<String, CmdPos> = HashMap::new();
        for f_name in files.iter() {
            let file = File::open(path.join(f_name))?;
            let mut reader = BufPosReader::new(file)?;
            //TODO: Remove unwraps, looks ugly af
            let f_name = f_name
                .find('-')
                .map(|i| Some(f_name[..i].to_owned()))
                .map(|name| {
                    name.unwrap()
                        .parse::<usize>()
                        .expect("file name not supported")
                })
                .expect("failed to parse file name");

            let file_sz = replay(&mut reader, &mut idx, f_name)?;
            total_sz += file_sz;
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
                .write(true)
                .open(path.join(file_name))?;
        } else {
            active_id = 0;
            active_file = std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .write(true)
                .create(true)
                .open(path.join("0-log.json"))?;

            let f = File::open(path.join("0-log.json"))?;
            let f = BufPosReader::new(f)?;
            readers.insert(0, f);
        }
        let mut writer = BufPosWriter::new(active_file)?;
        writer.seek(SeekFrom::End(0))?;
        let store = KvStore::new(writer, readers, idx, active_id, total_sz, path)?;
        Ok(store)
    }

    fn compact(&mut self) -> Result<()> {
        // To compact we first need to get a list of all files
        // then, for every file, we take its ID --> find it's reader, remove from map
        // close (if applicable)
        // close writer (& fush)
        // delete all files, create new file, replace writer
        self.active_id += 1;
        self.total_sz = 0;
        let file_path = log_path(&self.path, self.active_id);
        let f = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        self.writer = BufPosWriter::new(f)?;
        self.writer.seek(SeekFrom::End(0))?;

        for v in self.idx.values_mut() {
            let reader = self
                .readers
                .get_mut(&v.f_id)
                .expect("could not get reader associated with key");
            let mut buf = vec![0u8; v.sz];
            reader.seek(SeekFrom::Start(v.pos as u64))?;
            reader.read(&mut buf)?;
            let pos = self.writer.pos;
            let sz = self.writer.write(&buf)?;
            *v = CmdPos {
                f_id: self.active_id,
                pos,
                sz,
            };
            self.writer.flush()?;
            self.total_sz += sz;
        }

        let files = self
            .readers
            .keys()
            .map(|id| log_path(&self.path, *id))
            .collect::<Vec<PathBuf>>();

        self.readers = HashMap::new();
        let f = File::open(&file_path)?;
        let mut reader = BufPosReader::new(f)?;
        reader.seek(SeekFrom::Start(0))?;
        self.readers.insert(self.active_id, reader);

        for file in files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
}

fn replay(
    r: &mut BufPosReader<File>,
    idx: &mut HashMap<String, CmdPos>,
    f_id: usize,
) -> Result<usize> {
    let pos = r.seek(SeekFrom::Start(0))?;
    let mut pos = pos as usize;
    let mut stream = serde_json::Deserializer::from_reader(r).into_iter::<Command>();
    while let Some(value) = stream.next() {
        let value = value?;
        // offset represents how many bytes have been read so far
        let offset = stream.byte_offset();
        if let Command::Set { key, .. } = value {
            idx.insert(
                key,
                CmdPos {
                    f_id,
                    pos,
                    sz: offset - pos,
                },
            );
        } else if let Command::Rm { key } = value {
            idx.remove(&key);
        }
        pos = offset;
    }

    Ok(pos)
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

impl<W: Write + Seek> Seek for BufPosWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let bytes = self.writer.seek(pos)?;
        self.pos = bytes as usize;
        Ok(bytes)
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

// Credit to pingcap guide
fn log_path(dir: &Path, id: usize) -> PathBuf {
    dir.join(format!("{}-log.json", id))
}
