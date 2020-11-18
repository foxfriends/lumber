use super::*;
use crate::parser::Rule;
use std::collections::HashMap;

/// The body of a rule.
#[derive(Default, Clone, Debug)]
pub(crate) struct Body(pub(crate) Option<Disjunction>);

impl Body {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::body);
        Self::new_inner(just!(pair.into_inner()), context)
    }

    pub fn new_inner(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::disjunction);
        Some(Self(Some(Disjunction::new(pair, context)?)))
    }

    pub fn new_evaluation(terms: Vec<Unification>) -> Self {
        let terms = terms
            .into_iter()
            .map(|term| Procession { steps: vec![term] })
            .collect();
        Self(Some(Disjunction {
            cases: vec![Conjunction { terms }],
        }))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.0
            .iter_mut()
            .flat_map(|disjunction| disjunction.handles_mut())
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.0
            .iter()
            .flat_map(|disjunction| disjunction.identifiers())
    }

    pub fn check_variables(&self, head: &Query, context: &mut Context) {
        let counts = self
            .identifiers()
            .chain(head.identifiers())
            .filter(|ident| !ident.is_wildcard())
            .fold(
                HashMap::<Identifier, usize>::default(),
                |mut map, identifier| {
                    *map.entry(identifier).or_default() += 1;
                    map
                },
            );

        for (identifier, count) in counts {
            if count <= 1 {
                context.error_singleton_variable(head.as_ref(), identifier.name());
            }
        }
    }
}
