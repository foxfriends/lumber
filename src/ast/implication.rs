use super::*;

#[derive(Clone, Debug)]
pub struct Implication {
    conditions: Vec<Unification>,
}
