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

impl Variables for Body {
    fn variables(&self, vars: &mut Vec<Variable>) {
        self.0.variables(vars)
    }
}
