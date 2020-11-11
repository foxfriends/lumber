use super::Value;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;

/// An implementation of a list which may be incomplete, suitable for Lumber values which
/// may themselves be unbound.
#[derive(Clone, Debug)]
pub struct List {
    pub(crate) values: Vec<Option<Value>>,
    pub(crate) complete: bool,
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl List {
    pub(crate) fn new(values: Vec<Option<Value>>, complete: bool) -> Self {
        Self { values, complete }
    }

    /// Adds a value to this list.
    pub fn push<V>(&mut self, value: V)
    where
        Option<Value>: From<V>,
    {
        self.values.push(value.into());
    }
}

impl<V> FromIterator<V> for List
where
    Value: From<V>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        Self {
            values: iter.into_iter().map(Value::from).map(Some).collect(),
            complete: true,
        }
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
