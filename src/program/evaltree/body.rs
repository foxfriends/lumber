use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// The body of a rule.
#[derive(Default, Clone, Debug)]
pub(crate) struct Body(pub(crate) Disjunction);

impl Body {
    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.0.handles_mut()
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.0.identifiers()
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ast::Body> for Body {
    fn from(ast: ast::Body) -> Self {
        Self(Disjunction::from(ast.0))
    }
}
