use std::fmt;
use crate::code::Code;

#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub args: Vec<(String, String)>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {{ code: {}, message: {}", self.code, self.message)?;
        if !self.args.is_empty() {
            write!(f, ", args: {:?}", self.args)?;
        }
        write!(f, " }}")
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(code: i32) -> Self {
        Self {
            code,
            message: String::new(),
            args: vec![],
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    pub fn with_arg(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.args.push((key.into(), val.into()));
        self
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(Code::InternalServerError.as_i32()).with_message(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::new(Code::InternalServerError.as_i32()).with_message(err.to_string())
    }
}
