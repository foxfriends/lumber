use super::*;

/// An implication of unifications.
#[derive(Clone, Debug)]
pub struct Implication {
    /// Conditions which, assuming the former are satisfied, require the latter.
    conditions: Vec<Unification>,
}
