use super::Value;
use crate::ast::*;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Field {
    Index(usize),
    Name(String),
}

/// A Lumber structure, containing a combination of named and indexed fields.
#[derive(Clone, Debug)]
pub struct Struct {
    pub(crate) name: Atom,
    pub(crate) fields: HashMap<Field, Option<Value>>,
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
}
