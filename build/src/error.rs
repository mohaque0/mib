use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

#[derive(Debug, Clone, Fail)]
pub enum BuildError {
    ConfigError(String),
    ExecutionError(String),
    IOError(String),
    NoSuchModule(String),
    NoSuchTask(String)
}

impl BuildError {
    pub fn get_message(&self) -> &String {
        match self {
            BuildError::ConfigError(msg) => &msg,
            BuildError::ExecutionError(msg) => &msg,
            BuildError::IOError(msg) => &msg,
            BuildError::NoSuchModule(msg) => &msg,
            BuildError::NoSuchTask(msg) => &msg
        }
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (name, msg) = match &self {
            BuildError::ConfigError(msg) => ("ConfigError", msg),
            BuildError::ExecutionError(msg) => ("ExecutionError", msg),
            BuildError::IOError(msg) => ("IOError", msg),
            BuildError::NoSuchModule(msg) => ("NoSuchModule", msg),
            BuildError::NoSuchTask(msg) => ("NoSuchTask", msg)
        };
        write!(f, "{}: {}", name, msg)
    }
}