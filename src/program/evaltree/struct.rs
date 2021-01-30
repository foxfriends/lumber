use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// A named container, which can optionally contain a value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    pub(crate) name: Atom,
    /// The contents of the struct
    pub(crate) contents: Option<Box<Pattern>>,
}

impl Struct {
    pub fn from_parts(name: Atom, contents: Option<Box<Pattern>>) -> Self {
        Self { name, contents }
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.contents
            .iter()
            .flat_map(|pattern| pattern.identifiers())
    }

    pub fn identifiers_mut(&mut self) -> impl Iterator<Item = &mut Identifier> + '_ {
        self.contents
            .iter_mut()
            .flat_map(|pattern| pattern.identifiers_mut())
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        match &self.contents {
            None => Ok(()),
            Some(pattern) if pattern.is_container() => write!(f, " {}", pattern),
            Some(pattern) => write!(f, "({})", pattern),
        }
    }
}

impl From<ast::Struct> for Struct {
    fn from(ast: ast::Struct) -> Self {
        Self {
            name: ast.name,
            contents: ast.contents.map(|pat| Box::new(Pattern::from(*pat))),
        }
    }
}
