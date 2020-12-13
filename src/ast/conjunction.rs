use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A conjunction of processions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Conjunction {
    /// Terms between which variable bindings are shared.
    pub(crate) terms: Vec<Procession>,
}

impl Conjunction {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::conjunction);
        let terms = pair
            .into_inner()
            .map(|pair| Procession::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { terms })
    }

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Handle>>(&mut self, mut resolve: F) {
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
