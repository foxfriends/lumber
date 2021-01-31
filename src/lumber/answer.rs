use super::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;

/// An answer to a question. Contains a single valid binding of the named variables in the question
/// to values that satisfy the program.
#[derive(Clone)]
pub struct Answer {
    variables: HashMap<String, Option<Value>>,
}

impl Answer {
    /// Gets the value of a variable from this answer. Returns `None` if the variable is not
    /// bound or does not exist.
    pub fn get<Q>(&self, key: &Q) -> Option<&Value>
    where
        Q: Hash + Eq + ?Sized,
        String: Borrow<Q>,
    {
        self.variables.get(key)?.as_ref()
    }

    /// Removes a variable from the answer. Returns `None` if the variable is not
    /// bound or does not exist.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<Value>
    where
        Q: Hash + Eq + ?Sized,
        String: Borrow<Q>,
    {
        self.variables.remove(key)?
    }
}

impl FromIterator<(String, Option<Value>)> for Answer {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Option<Value>)>,
    {
        Self {
            variables: HashMap::from_iter(iter),
        }
    }
}

impl IntoIterator for Answer {
    type Item = <HashMap<String, Option<Value>> as IntoIterator>::Item;
    type IntoIter = <HashMap<String, Option<Value>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.variables.into_iter()
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.variables.is_empty() {
            return write!(f, "true");
        }
        for (i, (var, val)) in self.variables.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match val {
                Some(val) => write!(f, "{} = {}", var, val)?,
                None => write!(f, "{} = _", var)?,
            }
        }
        Ok(())
    }
}
