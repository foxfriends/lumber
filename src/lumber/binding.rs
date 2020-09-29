use super::Value;
use std::collections::HashMap;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Clone, Debug)]
pub struct Binding(HashMap<String, Option<Value>>);
