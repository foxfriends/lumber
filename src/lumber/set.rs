use super::Value;

/// An implementation of sets suitable for Lumber values which may be unbound.
#[derive(Clone, Debug)]
pub struct Set {
    values: Vec<Value>,
    min_unbound: Option<usize>,
    max_unbound: Option<usize>,
}
