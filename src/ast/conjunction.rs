use super::*;
use crate::parser::Rule;

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

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.terms.iter_mut().flat_map(|term| term.handles_mut())
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.terms.iter().flat_map(|term| term.identifiers())
    }
}
