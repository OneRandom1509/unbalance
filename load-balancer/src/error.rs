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

#[derive(Debug)]
pub enum WorkerError {
    CannotConnectToWorker(String),
    Other(String),
}

impl fmt::Display for ThreadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadError::InvalidSize(msg) => write!(f, "InvalidSizeError: {}", msg),
            ThreadError::MutexError(msg) => write!(f, "MutexError: {}", msg),
            ThreadError::SenderError(msg) => write!(f, "SenderThreadError: {}", msg),
            ThreadError::ThreadHandlerError(msg) => write!(f, "ThreadHandlerError: {}", msg),
            ThreadError::JoinError(msg) => write!(f, "JoinThreadError: {}", msg),
            ThreadError::Other(msg) => write!(f, "OtherThreadError: {}", msg),
        }
    }
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerError::CannotConnectToWorker(msg) => write!(f, "CannotConnectError: {}", msg),
            WorkerError::Other(msg) => write!(f, "OtherWorkerError: {}", msg),
        }
    }
}

impl Error for ThreadError {}

impl Error for WorkerError {}

impl From<Box<dyn Error + Send + Sync>> for ThreadError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        ThreadError::Other(error.to_string())
    }
}

impl From<Box<dyn Error + Send + Sync>> for WorkerError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        WorkerError::Other(error.to_string())
    }
}
