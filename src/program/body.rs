use super::*;

/// The body of a rule.
#[derive(Clone, Debug)]
pub struct Body {
    /// Steps between which variable bindings should not be backtracked.
    steps: Vec<Disjunction>,
}
