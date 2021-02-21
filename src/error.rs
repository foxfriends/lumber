use crate::ast::Scope;
use crate::Question;
#[cfg(feature = "serde")]
use serde::{de, ser};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};

/// The kind of error that has occurred.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    /// The error occurred while interacting with the file system.
    Io,
    /// An error was encountered in the source file, preventing the program from being created.
    /// This error likely cannot be handled programmatically.
    Parse,
    /// An error has occurred while attempting to extract a pattern from a binding.
    Binding,
    /// One or more of the unit tests defined in the source code have failed.
    Test,
    /// Contains multiple errors of various sources. This error can be printed to the user to
    /// help with debugging. This error likely cannot be handled programmatically.
    Multiple,
    /// An error has occurred during serialization of a Rust value to a Lumber value.
    #[cfg(feature = "serde")]
    Ser,
    /// An error has occurred during deserialization of a Lumber value to a Rust value.
    #[cfg(feature = "serde")]
    De,
}

/// An error that has occurred within Lumber.
#[derive(Debug)]
pub struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) message: String,
    pub(crate) source: Option<Box<dyn std::error::Error + 'static>>,
}

impl Error {
    pub(crate) fn parse<S: ?Sized + ToOwned<Owned = String>>(message: &S) -> Self
    where
        String: std::borrow::Borrow<S>,
    {
        Self {
            kind: ErrorKind::Parse,
            message: message.to_owned(),
            source: None,
        }
    }

    pub(crate) fn binding<S: ?Sized + ToOwned<Owned = String>>(message: &S) -> Self
    where
        String: std::borrow::Borrow<S>,
    {
        Self {
            kind: ErrorKind::Binding,
            message: message.to_owned(),
            source: None,
        }
    }

    pub(crate) fn test(tests: Vec<Question>) -> Self {
        let count = tests.len();
        let tests = tests
            .into_iter()
            .map(|question| format!("\t{}", question))
            .collect::<Vec<_>>()
            .join("\n");
        let message = format!("{} tests have failed:\n{}", count, tests);
        Self {
            kind: ErrorKind::Test,
            message,
            source: None,
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn ser<S: Display>(message: S) -> Self {
        Self {
            kind: ErrorKind::Ser,
            message: message.to_string(),
            source: None,
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn de<S: Display>(message: S, path: &[String]) -> Self {
        Self {
            kind: ErrorKind::De,
            message: if path.is_empty() {
                message.to_string()
            } else {
                format!("{} at {}", message, path.join(" -> "))
            },
            source: None,
        }
    }

    pub(crate) fn multiple_by_module(errors: HashMap<Scope, Vec<Self>>) -> Self {
        Self {
            kind: ErrorKind::Multiple,
            message: errors
                .into_iter()
                .map(|(scope, errors)| {
                    let mut message = String::new();
                    if errors.is_empty() {
                        return message;
                    }
                    writeln!(message, "-- {} errors in module {} --", errors.len(), scope).unwrap();
                    for error in &errors {
                        writeln!(message, "\t{}", error).unwrap();
                    }
                    message
                })
                .collect::<String>(),
            source: None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|err| err.as_ref())
    }
}

#[cfg(feature = "serde")]
impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::ser(msg)
    }
}

#[cfg(feature = "serde")]
impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self {
            kind: ErrorKind::De,
            message: msg.to_string(),
            source: None,
        }
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
