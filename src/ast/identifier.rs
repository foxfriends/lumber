use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A unique identifier for a variable.
///
/// Note that the original name of the variable is stored elsewhere, as it is not relevant
/// to the computation but is useful in output and debugging.
#[derive(Clone, Eq, Debug)]
pub struct Identifier(Rc<String>, bool);

impl Hash for Identifier {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        (Rc::as_ptr(&self.0) as usize).hash(hasher)
    }
}

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
        Self(Rc::new(name), false)
    }

    pub(crate) fn wildcard<S: Into<String>>(name: S) -> Self {
        Self(Rc::new(name.into()), true)
    }

    pub fn name(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_wildcard(&self) -> bool {
        self.1
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
