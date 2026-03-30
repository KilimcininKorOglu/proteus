use std::fmt;

pub type ProteusResult<T> = Result<T, ProteusError>;

#[derive(Debug)]
pub enum ProteusError {
    Io(std::io::Error),
    InvalidArgument(String),
    Other(String),
}

impl fmt::Display for ProteusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProteusError::Io(e) => write!(f, "{e}"),
            ProteusError::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
            ProteusError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProteusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProteusError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProteusError {
    fn from(e: std::io::Error) -> Self {
        ProteusError::Io(e)
    }
}
