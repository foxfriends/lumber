use super::Value;

/// An implementation of a list which may be incomplete, suitable for Lumber values which
/// may themselves be unbound.
#[derive(Clone, Debug)]
pub struct List {
    pub(crate) values: Vec<Option<Value>>,
    pub(crate) complete: bool,
}

impl List {
    pub(crate) fn new(values: Vec<Option<Value>>, complete: bool) -> Self {
        Self { values, complete }
    }
}
