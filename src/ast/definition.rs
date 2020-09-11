use super::*;
use std::collections::HashMap;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Debug)]
pub struct Definition(HashMap<Query, Body>);
