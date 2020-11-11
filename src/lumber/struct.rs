use super::Value;
use crate::ast::*;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops::{Index, IndexMut};

/// A Lumber structure, containing a combination of named and indexed fields. Atoms in Lumber are
/// the same as structs with no fields.
#[derive(Clone, PartialEq, Debug)]
pub struct Struct {
    pub(crate) name: Atom,
    pub(crate) values: Vec<Option<Value>>,
    pub(crate) fields: HashMap<Atom, Vec<Option<Value>>>,
}

impl Struct {
    pub(crate) fn new(
        name: Atom,
        values: Vec<Option<Value>>,
        fields: HashMap<Atom, Vec<Option<Value>>>,
    ) -> Self {
        Struct {
            name,
            values,
            fields,
        }
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
            values: vec![],
            fields: HashMap::new(),
        }
    }

    /// Adds an unnamed value to the end of this Struct.
    pub fn push(&mut self, value: Option<Value>) {
        self.values.push(value);
    }

    /// Adds an unnamed value to the end of this Struct.
    pub fn with(mut self, value: Option<Value>) -> Self {
        self.values.push(value);
        self
    }

    /// Sets the values of a named field in this Struct.
    ///
    /// Until tuples are properly implemented, this function takes a `Vec` because struct
    /// fields may have multiple values.
    pub fn set(&mut self, field: impl AsRef<str>, value: Vec<Option<Value>>) {
        self.fields.insert(Atom::from(field.as_ref()), value);
    }

    /// Sets the values of a named field in this Struct.
    ///
    /// Until tuples are properly implemented, this function takes a `Vec` because struct
    /// fields may have multiple values.
    pub fn with_entry(mut self, field: impl AsRef<str>, value: Vec<Option<Value>>) -> Self {
        self.fields.insert(Atom::from(field.as_ref()), value);
        self
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
        self.values.is_empty() && self.fields.is_empty()
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

impl Index<usize> for Struct {
    type Output = Option<Value>;
    fn index(&self, index: usize) -> &Self::Output {
        self.values.index(index)
    }
}

impl IndexMut<usize> for Struct {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.values.index_mut(index)
    }
}

impl Index<String> for Struct {
    type Output = Vec<Option<Value>>;
    fn index(&self, index: String) -> &Self::Output {
        self.fields.index(&Atom::from(index))
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        if !self.fields.is_empty() {
            write!(f, "(")?;
            for (i, value) in self.values.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                match value {
                    Some(value) => value.fmt(f)?,
                    None => write!(f, "_")?,
                }
            }
            for (i, (name, values)) in self.fields.iter().enumerate() {
                if self.values.len() != 0 || i != 0 {
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
