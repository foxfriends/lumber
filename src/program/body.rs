use super::*;
use crate::parser::Rule;

/// The body of a rule.
#[derive(Default, Clone, Debug)]
pub struct Body {
    /// Steps between which variable bindings should not be backtracked.
    steps: Vec<Disjunction>,
}

impl Body {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::body);
        Self::new_inner(pair, context)
    }

    pub(crate) fn new_inner(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        let pair = just!(Rule::procession, pair.into_inner());
        let steps = pair
            .into_inner()
            .map(|pair| Disjunction::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { steps })
    }

    pub(crate) fn new_evaluation(unifications: Vec<Unification>) -> Self {
        let case = Conjunction {
            terms: unifications
                .into_iter()
                .map(|unification| Implication {
                    conditions: vec![unification],
                })
                .collect(),
        };
        let step = Disjunction { cases: vec![case] };
        Body { steps: vec![step] }
    }

    pub(crate) fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        todo!();
        std::iter::empty()
    }
}
