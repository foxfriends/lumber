use super::*;
use crate::parser::Rule;

/// An implication of unifications.
#[derive(Clone, Debug)]
pub struct Implication {
    /// Conditions which, assuming the former are satisfied, require the latter.
    pub(super) conditions: Vec<Unification>,
}

impl Implication {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::implication);
        let conditions = pair
            .into_inner()
            .map(|pair| Unification::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { conditions })
    }

    pub(crate) fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.conditions
            .iter_mut()
            .flat_map(|unification| unification.handles_mut())
    }

    pub(crate) fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.conditions
            .iter()
            .flat_map(|unification| unification.identifiers())
    }
}
