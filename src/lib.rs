//! Lumber is a logic programming language, mainly intended to be embedded in Rust programs.
//!
//! More info will be available soon.

#![feature(bindings_after_at, generators, generator_trait)]

#[macro_use]
mod lumber;
mod ast;
mod core;
mod error;
mod parser;
mod program;

#[cfg(feature = "serde")]
pub mod de;
#[cfg(feature = "serde")]
pub mod ser;

type Pairs<'i> = pest::iterators::Pairs<'i, parser::Rule>;
type Pair<'i> = pest::iterators::Pair<'i, parser::Rule>;

pub use crate::lumber::*;
pub use error::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test;
