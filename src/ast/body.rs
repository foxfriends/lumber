use super::*;
use crate::parser::Rule;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// The body of a rule.
#[derive(Default, Clone, Debug)]
pub(crate) struct Body(pub(crate) Disjunction);

impl Body {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::body);
        Self::new_inner(just!(pair.into_inner()), context)
    }

    pub fn new_inner(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::disjunction);
        Some(Self(Disjunction::new(pair, context)?))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.0.handles_mut()
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.0.identifiers()
    }

    pub fn check_variables(&self, head: &Head, context: &mut Context) {
        let counts = self.identifiers().chain(head.identifiers()).fold(
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

    pub fn resolve_handles<F: FnMut(&Handle) -> Option<Handle>>(&mut self, resolve: &mut F) {
        self.handles_mut().for_each(move |handle| {
            if let Some(resolved) = resolve(handle) {
                *handle = resolved;
            }
        });
    }

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Operator>>(&mut self, resolve: &mut F) {
        self.0.resolve_operators(resolve)
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
