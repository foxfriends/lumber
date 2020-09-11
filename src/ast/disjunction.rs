use super::*;

/// A disjunction of conjunctions.
#[derive(Clone, Debug)]
pub struct Disjunction {
    /// Cases between which variable bindings are not shared.
    cases: Vec<Conjunction>,
}
