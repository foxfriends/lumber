use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// A conjunction of processions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Conjunction {
    /// Terms between which variable bindings are shared.
    pub(crate) terms: Vec<Procession>,
}

impl Conjunction {
    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.terms.iter_mut().flat_map(|term| term.handles_mut())
    }
}

impl Display for Conjunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, term) in self.terms.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            term.fmt(f)?;
        }
        Ok(())
    }
}

impl From<ast::Conjunction> for Conjunction {
    fn from(ast: ast::Conjunction) -> Self {
        Self {
            terms: ast.terms.into_iter().map(Procession::from).collect(),
        }
    }
}

impl Variables for Conjunction {
    fn variables(&self, vars: &mut Vec<Variable>) {
        for term in &self.terms {
            term.variables(vars);
        }
    }
}
