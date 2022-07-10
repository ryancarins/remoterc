use std::error;
use std::fmt;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum CommunicationError {
    Tungstenite(tungstenite::Error),
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommunicationError::Tungstenite(ref err) => write!(f, "Tungstenite error: {}", err),
        }
    }
}

impl error::Error for CommunicationError {
    fn description(&self) -> &str {
        match *self {
            CommunicationError::Tungstenite(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            CommunicationError::Tungstenite(ref err) => Some(err),
        }
    }
}

impl From<tungstenite::Error> for CommunicationError {
    fn from(err: tungstenite::Error) -> Self {
        CommunicationError::Tungstenite(err)
    }
}
