use super::*;

#[derive(Clone, Debug)]
pub struct Conjunction {
    terms: Vec<Implication>,
}
