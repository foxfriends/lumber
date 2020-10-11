use std::cmp::{Ord, Ordering, PartialOrd};
use std::rc::Rc;

/// A unique identifier for a variable.
///
/// Note that the original name of the variable is stored elsewhere, as it is not relevant
/// to the computation but is useful in output and debugging.
#[derive(Clone, Hash, Eq, Debug)]
pub struct Identifier(Rc<String>);

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        Rc::as_ptr(&self.0).cmp(&Rc::as_ptr(&other.0))
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Identifier {
    pub(crate) fn new(name: String) -> Self {
        Self(Rc::new(name))
    }

    pub fn name(&self) -> &str {
        self.0.as_str()
    }
}
