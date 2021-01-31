use super::Value;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;

/// An answer to a question. Contains a single valid binding of the named variables in the question
/// to values that satisfy the program.
#[derive(Clone)]
pub struct Answer {
    variables: HashMap<String, Option<Value>>,
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
