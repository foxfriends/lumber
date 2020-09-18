use super::*;
use std::collections::HashMap;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Default, Clone, Debug)]
pub struct Definition(HashMap<Query, Body>);

impl Definition {
    pub(crate) fn insert(&mut self, query: Query, body: Body) {
        self.0.insert(query, body);
    }

    pub(crate) fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.0.values_mut()
    }
}
