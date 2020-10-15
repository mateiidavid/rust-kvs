use std::fmt;

#[derive(Debug)]
/// A union of all possible errors in our lib
pub enum KvStoreError {
    // Errors from ext libs
    Io(std::io::Error),
    Serde(serde_json::Error),
    // Errors from this lib
    Store(ErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, PartialEq, Ord)]
pub enum ErrorKind {
    NotFound,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match self {
            ErrorKind::NotFound => "Key not found",
        }
    }
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvStoreError::Io(err) => err.fmt(f),
            KvStoreError::Serde(err) => err.fmt(f),
            KvStoreError::Store(err) => write!(f, "KvStore error occurred {:?}", err),
        }
    }
}

// Enable ? op
impl From<std::io::Error> for KvStoreError {
    fn from(err: std::io::Error) -> KvStoreError {
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
