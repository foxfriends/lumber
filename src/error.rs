use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    Io,
    Parse,
}

#[derive(Debug)]
pub struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) message: String,
    pub(crate) source: Option<Box<dyn std::error::Error + 'static>>,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|err| err.as_ref())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io,
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

impl<R: pest::RuleType + 'static> From<pest::error::Error<R>> for Error {
    fn from(error: pest::error::Error<R>) -> Self {
        Self {
            kind: ErrorKind::Parse,
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}
