use super::Value;
use std::fmt::{self, Display, Formatter};

/// An implementation of a set which may be incomplete, suitable for Lumber values which
/// may themselves be unbound.
#[derive(Clone, Debug)]
pub struct Set {
    pub(crate) values: Vec<Option<Value>>,
    pub(crate) complete: bool,
}

impl Set {
    pub(crate) fn new(values: Vec<Option<Value>>, complete: bool) -> Self {
        Self { values, complete }
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut values = self.values.iter();
        for (i, value) in self.values.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match value {
                Some(value) => value.fmt(f)?,
                None => write!(f, "_")?,
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}
