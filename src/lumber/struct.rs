use super::Value;
use crate::ast::*;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Field {
    Unnamed,
    Named(Atom),
}

/// A Lumber structure, containing a combination of named and indexed fields. Atoms in Lumber are
/// the same as structs with no fields.
#[derive(Clone, PartialEq, Debug)]
pub struct Struct {
    pub(crate) name: Atom,
    pub(crate) fields: HashMap<Field, Vec<Option<Value>>>,
}

impl Struct {
    pub(crate) fn new(name: Atom, arity: &Arity, mut values: Vec<Option<Value>>) -> Self {
        let mut fields = HashMap::new();
        if arity.len != 0 {
            fields.insert(
                Field::Unnamed,
                values.drain(0..arity.len as usize).collect(),
            );
        }
        for field in arity.fields() {
            fields.insert(
                Field::Named(field.name.clone()),
                values.drain(0..field.len as usize).collect(),
            );
        }
        Struct { name, fields }
    }

    /// Constructs an atom.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert!(atom.is_atom());
    /// ```
    pub fn atom(name: impl Into<String>) -> Self {
        Self {
            name: Atom::from(name.into()),
            fields: HashMap::new(),
        }
    }

    /// Checks if this struct is actually just an atom. An atom is a struct with no fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert!(atom.is_atom());
    /// ```
    pub fn is_atom(&self) -> bool {
        self.fields.is_empty()
    }

    /// The name or symbol of this struct or atom.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert_eq!(atom.name(), "hello");
    /// ```
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        if !self.fields.is_empty() {
            write!(f, "(")?;
            let empty = vec![];
            let unnamed_fields = self.fields.get(&Field::Unnamed).unwrap_or(&empty);
            for (i, value) in unnamed_fields.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                match value {
                    Some(value) => value.fmt(f)?,
                    None => write!(f, "_")?,
                }
            }
            for (i, (name, values)) in self
                .fields
                .iter()
                .filter_map(|(k, v)| match k {
                    Field::Unnamed => None,
                    Field::Named(name) => Some((name, v)),
                })
                .enumerate()
            {
                if unnamed_fields.len() != 0 || i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: ", name)?;
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
            write!(f, ")")?;
        }
        Ok(())
    }
}
