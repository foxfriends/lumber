use super::Value;
use std::fmt::{self, Display, Formatter};

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

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, value) in self.values.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match value {
                Some(value) => value.fmt(f)?,
                None => write!(f, "_")?,
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}
