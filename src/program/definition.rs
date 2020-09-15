use super::*;
use std::collections::HashMap;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Default, Clone, Debug)]
pub struct Definition(HashMap<Query, Body>);

impl Definition {
    pub fn insert(&mut self, query: Query, body: Body) {
        self.0.insert(query, body);
    }
}
