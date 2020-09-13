use super::*;
use std::collections::HashMap;
use std::rc::Rc;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Clone, Debug)]
pub struct Definition(Rc<HashMap<Query, Body>>);
