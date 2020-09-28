use super::Value;
use crate::ast::*;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum Field {
    Index(usize),
    Name(String),
}

/// A Lumber structure, containing a combination of named and indexed fields.
#[derive(Clone, Debug)]
pub struct Struct {
    name: Atom,
    fields: HashMap<Field, Option<Value>>,
}
