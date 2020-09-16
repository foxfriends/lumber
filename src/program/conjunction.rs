use super::*;
use crate::parser::Rule;

/// A conjunction of implications.
#[derive(Clone, Debug)]
pub struct Conjunction {
    /// Terms between which variable bindings are shared.
    pub(super) terms: Vec<Implication>,
}

impl Conjunction {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::conjunction);
        let terms = pair
            .into_inner()
            .map(|pair| Implication::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { terms })
    }
}
