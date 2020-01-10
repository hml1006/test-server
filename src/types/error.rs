use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    UnsupportedMethod,
}

impl ErrorKind {
    pub fn description(&self) -> &str {
        match self {
            ErrorKind::UnsupportedMethod => "Unsupported Method"
        }
    }
}

impl ToString for ErrorKind {
    fn to_string(&self) -> std::string::String {
        self.description().to_string()
    }
}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorKind,
}

impl Error {
    pub fn new(error_type: ErrorKind) -> Self {
        Error { error_type}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type.description())
    }
}

impl StdError for Error {}
