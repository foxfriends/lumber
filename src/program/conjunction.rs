use super::*;

/// A conjunction of implications.
#[derive(Clone, Debug)]
pub struct Conjunction {
    /// Terms between which variable bindings are shared.
    terms: Vec<Implication>,
}
