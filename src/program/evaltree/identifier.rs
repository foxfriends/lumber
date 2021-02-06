use crate::ast;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A unique identifier for a variable.
///
/// Note that the original name of the variable is stored elsewhere, as it is not relevant
/// to the computation but is useful in output and debugging.
#[derive(Clone, Eq, Debug)]
pub struct Identifier {
    name: Rc<String>,
    is_wildcard: bool,
}

impl Hash for Identifier {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        (Rc::as_ptr(&self.name) as usize).hash(hasher)
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.name, &other.name)
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        Rc::as_ptr(&self.name).cmp(&Rc::as_ptr(&other.name))
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Identifier {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name: Rc::new(name),
            is_wildcard: false,
        }
    }

    pub(crate) fn wildcard<S: Into<String>>(name: S) -> Self {
        Self {
            name: Rc::new(name.into()),
            is_wildcard: true,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_wildcard(&self) -> bool {
        self.is_wildcard
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl From<ast::Identifier> for Identifier {
    fn from(ast: ast::Identifier) -> Self {
        Self {
            name: ast.0,
            is_wildcard: false,
        }
    }
}
