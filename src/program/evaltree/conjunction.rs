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
    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Operator>>(&mut self, mut resolve: F) {
        self.terms
            .iter_mut()
            .for_each(move |term| term.resolve_operators(&mut resolve))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.terms.iter_mut().flat_map(|term| term.handles_mut())
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.terms.iter().flat_map(|term| term.identifiers())
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
