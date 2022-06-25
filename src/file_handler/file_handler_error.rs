use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum FileHandlerError {
    Io(io::Error),
    Jwalk(jwalk::Error),
    Regex(regex::Error),
}

impl fmt::Display for FileHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileHandlerError::Io(ref err) => write!(f, "IO Error: {}", err),
            FileHandlerError::Jwalk(ref err) => write!(f, "IO Error: {}", err),
            FileHandlerError::Regex(ref err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl error::Error for FileHandlerError {
    fn description(&self) -> &str {
        match *self {
            FileHandlerError::Io(ref err) => err.description(),
            FileHandlerError::Jwalk(ref err) => err.description(),
            FileHandlerError::Regex(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            FileHandlerError::Io(ref err) => Some(err),
            FileHandlerError::Jwalk(ref err) => Some(err),
            FileHandlerError::Regex(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for FileHandlerError {
    fn from(err: io::Error) -> Self {
        FileHandlerError::Io(err)
    }
}

impl From<jwalk::Error> for FileHandlerError {
    fn from(err: jwalk::Error) -> Self {
        FileHandlerError::Jwalk(err)
    }
}

impl From<regex::Error> for FileHandlerError {
    fn from(err: regex::Error) -> Self {
        FileHandlerError::Regex(err)
    }
}