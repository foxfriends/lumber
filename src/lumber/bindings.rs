use super::Value;
use std::collections::HashMap;

/// A set of bindings, which may or may not be bound.
#[derive(Clone, Debug)]
pub struct Bindings(HashMap<String, Option<Value>>);
