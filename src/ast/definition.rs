use super::*;
use std::iter::FromIterator;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum RuleKind {
    Multi,
    Once,
}

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Default, Clone, Debug)]
pub(crate) struct Definition(Vec<(Query, RuleKind, Body)>);

impl Definition {
    pub fn insert(&mut self, query: Query, kind: RuleKind, body: Body) {
        self.0.push((query, kind, body));
    }

    pub fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.0.iter_mut().map(|(_, _, body)| body)
    }

    pub fn merge(&mut self, mut other: Definition) {
        self.0.append(&mut other.0);
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Query, RuleKind, Body)> {
        self.0.iter()
    }
}

impl FromIterator<Self> for Definition {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        iter.into_iter().fold(Self::default(), |mut acc, def| {
            acc.merge(def);
            acc
        })
    }
}
