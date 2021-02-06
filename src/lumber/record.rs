use super::Value;
use crate::program::evaltree::*;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// An implementation of a record which may be incomplete, suitable for Lumber values which
/// may themselves be unbound.
#[derive(Clone, Debug)]
pub struct Record {
    pub(crate) fields: HashMap<Atom, Option<Value>>,
    pub(crate) complete: bool,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            fields: HashMap::default(),
            complete: true,
        }
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl Record {
    /// Creates a new Lumber record value from a map of possibly unbound Values.
    pub fn new(fields: HashMap<String, Option<Value>>) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(key, value)| (Atom::from(key), value))
                .collect(),
            complete: true,
        }
    }

    /// Adds a field to this record.
    pub fn with(mut self, key: impl AsRef<str>, value: Option<Value>) -> Self {
        self.fields.insert(Atom::from(key.as_ref()), value);
        self
    }

    /// Sets a field of this record.
    pub fn set(&mut self, key: impl AsRef<str>, value: Option<Value>) {
        self.fields.insert(Atom::from(key.as_ref()), value);
    }

    /// Iterates over the entries of this record.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Option<Value>)> {
        self.fields.iter().map(|(key, value)| (key.as_ref(), value))
    }

    /// Iterates over the entries of this record, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut Option<Value>)> {
        self.fields
            .iter_mut()
            .map(|(key, value)| (key.as_ref(), value))
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.fields.is_empty() {
            write!(f, "{{:}}")
        } else {
            write!(f, "{{ ")?;
            let mut keys_sorted: Vec<_> = self.fields.keys().collect();
            keys_sorted.sort();
            for (i, key) in keys_sorted.iter().enumerate() {
                let value = self.fields.get(key).unwrap();
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: ", key)?;
                match value {
                    Some(value) => value.fmt(f)?,
                    None => write!(f, "_")?,
                }
            }
            write!(f, " }}")?;
            Ok(())
        }
    }
}
