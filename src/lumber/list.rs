use super::Value;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

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

    /// Gets the number of elements in the list. This does not include the unknown elements if the list
    /// is incomplete.
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Index<usize> for List {
    type Output = Option<Value>;
    fn index(&self, index: usize) -> &Self::Output {
        self.values.index(index)
    }
}

impl IndexMut<usize> for List {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.values.index_mut(index)
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
