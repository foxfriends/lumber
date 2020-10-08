use super::Value;
use crate::ast::*;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Field {
    Index(usize),
    Name(String),
}

/// A Lumber structure, containing a combination of named and indexed fields. Atoms in Lumber are
/// the same as structs with no fields.
#[derive(Clone, Debug)]
pub struct Struct {
    pub(crate) name: Atom,
    pub(crate) fields: Vec<(Field, Option<Value>)>,
}

impl Struct {
    pub(crate) fn new(name: Atom, arity: &[Arity], values: Vec<Option<Value>>) -> Self {
        let fields = arity
            .into_iter()
            .flat_map(|arity| arity.iter())
            .enumerate()
            .map(|(i, name)| match name {
                Some(name) => Field::Name(name.to_owned()),
                None => Field::Index(i),
            })
            .zip(values.into_iter())
            .collect();
        Struct { name, fields }
    }

    /// Checks if this struct is actually just an atom. An atom is a struct with no fields.
    pub fn is_atom(&self) -> bool {
        self.fields.is_empty()
    }

    /// The name or symbol of this struct or atom.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        if !self.fields.is_empty() {
            write!(f, "(")?;
            for (i, (field, value)) in self.fields.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                match field {
                    Field::Index(..) => {}
                    Field::Name(name) => write!(f, "{}:", name)?,
                }
                match value {
                    Some(value) => value.fmt(f)?,
                    None => write!(f, "_")?,
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}
