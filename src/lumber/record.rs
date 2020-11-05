use super::Value;
use crate::ast::*;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// An implementation of a record which may be incomplete, suitable for Lumber values which
/// may themselves be unbound.
#[derive(Clone, Debug)]
pub struct Record {
    pub(crate) fields: HashMap<Atom, Vec<Option<Value>>>,
    pub(crate) complete: bool,
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl Record {
    pub(crate) fn new(fields: HashMap<Atom, Vec<Option<Value>>>, complete: bool) -> Self {
        Self { fields, complete }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (key, values)) in self.fields.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: ", key)?;
            for (i, value) in values.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                match value {
                    Some(value) => value.fmt(f)?,
                    None => write!(f, "_")?,
                }
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}
