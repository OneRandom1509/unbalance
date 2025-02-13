use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ThreadError {
    InvalidSize(String),
    MutexError(String),
    SenderError(String),
    ThreadHandlerError(String),
    JoinError(String),
    Other(String),
}

impl fmt::Display for ThreadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadError::InvalidSize(msg) => write!(f, "InvalidSizeError: {}", msg),
            ThreadError::MutexError(msg) => write!(f, "MutexError: {}", msg),
            ThreadError::SenderError(msg) => write!(f, "SenderError: {}", msg),
            ThreadError::ThreadHandlerError(msg) => write!(f, "ThreadHandlerError: {}", msg),
            ThreadError::JoinError(msg) => write!(f, "JoinError: {}", msg),
            ThreadError::Other(msg) => write!(f, "OtherError: {}", msg),
        }
    }
}

impl Error for ThreadError {}

impl From<Box<dyn Error + Send + Sync>> for ThreadError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        ThreadError::Other(error.to_string())
    }
}
