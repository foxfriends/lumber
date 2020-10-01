//! Lumber is a logic programming language, mainly intended to be embedded in Rust programs.
//!
//! More info will be available soon.

#![feature(exact_size_is_empty)]

mod ast;
mod core;
mod error;
mod lumber;
mod parser;
mod program;

type Pairs<'i> = pest::iterators::Pairs<'i, parser::Rule>;
type Pair<'i> = pest::iterators::Pair<'i, parser::Rule>;

pub use crate::lumber::*;
pub use error::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Error>;
