use super::*;

#[derive(Clone, Debug)]
pub struct Disjunction {
    cases: Vec<Conjunction>,
}
