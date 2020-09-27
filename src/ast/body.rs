use super::*;
use crate::parser::Rule;
use std::collections::HashMap;

/// The body of a rule.
#[derive(Default, Clone, Debug)]
pub(crate) struct Body {
    /// Steps between which variable bindings should not be backtracked.
    steps: Vec<Disjunction>,
}

impl Body {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::body);
        Self::new_inner(just!(pair.into_inner()), context)
    }

    pub fn new_inner(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::procession);
        let steps = pair
            .into_inner()
            .map(|pair| Disjunction::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { steps })
    }

    pub fn new_evaluation(unifications: Vec<Unification>) -> Self {
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

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.steps.iter_mut().flat_map(|step| step.handles_mut())
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.steps.iter().flat_map(|step| step.identifiers())
    }

    pub fn check_variables(&self, head: &Query, context: &mut Context) {
        let counts = self.identifiers().chain(head.identifiers()).fold(
            HashMap::<Identifier, usize>::default(),
            |mut map, identifier| {
                *map.entry(identifier).or_default() += 1;
                map
            },
        );

        for (identifier, count) in counts {
            let variable = context.name_identifier(identifier);
            if count <= 1 {
                let name = variable.to_owned();
                context.error_singleton_variable(head.as_ref(), name.as_str());
            }
        }
    }
}
