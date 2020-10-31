use std::fmt;

#[derive(Debug)]
/// A union of all possible errors in our lib
pub enum KvsError {
    // Errors from ext libs
    Io(std::io::Error),
    Serde(serde_json::Error),
    // Errors from this lib
    Store(ErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, PartialEq, Ord)]
pub enum ErrorKind {
    NotFound,
    UnsupportedCommand,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::NotFound => "Key not found",
            ErrorKind::UnsupportedCommand => "command is not supported",
        }
    }
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvsError::Io(err) => err.fmt(f),
            KvsError::Serde(err) => err.fmt(f),
            KvsError::Store(err) => write!(f, "store error occurred {:?}", err),
        }
    }
}

// Enable ? op
impl From<std::io::Error> for KvsError {
    fn from(err: std::io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}
///
pub type Result<T> = std::result::Result<T, KvsError>;
