use super::*;
use crate::parser::Rule;

/// A disjunction of conjunctions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Disjunction {
    /// Cases between which variable bindings are not shared.
    pub(crate) cases: Vec<Conjunction>,
}

impl Disjunction {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::disjunction);
        let cases = pair
            .into_inner()
            .map(|pair| Conjunction::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { cases })
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.cases.iter_mut().flat_map(|case| case.handles_mut())
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.cases.iter().flat_map(|case| case.identifiers())
    }
}
