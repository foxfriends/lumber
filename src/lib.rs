mod core;
mod parser;
mod program;

pub use program::Program;

mod error;

pub use error::{Error, ErrorKind};
pub type Result<T> = std::result::Result<T, Error>;

type Pairs<'i> = pest::iterators::Pairs<'i, parser::Rule>;
type Pair<'i> = pest::iterators::Pair<'i, parser::Rule>;
