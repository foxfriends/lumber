use super::*;
use crate::parser::Rule;

/// A disjunction of conjunctions.
#[derive(Clone, Debug)]
pub struct Disjunction {
    /// Cases between which variable bindings are not shared.
    pub(super) cases: Vec<Conjunction>,
}

impl Disjunction {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::disjunction);
        let cases = pair
            .into_inner()
            .map(|pair| Conjunction::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { cases })
    }
}
