use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// A named container, which can optionally contain a value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    pub(crate) name: Atom,
    /// The contents of the struct
    pub(crate) contents: Option<Pattern>,
}

impl Struct {
    pub fn from_parts(name: Atom, contents: Option<Pattern>) -> Self {
        Self { name, contents }
    }

    pub fn variables(&self, generation: usize) -> impl Iterator<Item = Variable> + '_ {
        self.contents
            .iter()
            .flat_map(move |pattern| pattern.variables(generation))
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        match &self.contents {
            None => Ok(()),
            Some(pattern) if pattern.kind().is_container() => write!(f, " {}", pattern),
            Some(pattern) => write!(f, "({})", pattern),
        }
    }
}

impl From<ast::Struct> for Struct {
    fn from(ast: ast::Struct) -> Self {
        Self {
            name: ast.name,
            contents: ast.contents.map(|pat| Pattern::from(*pat)),
        }
    }
}
