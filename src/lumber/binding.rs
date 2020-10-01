use super::Value;
use std::collections::HashMap;
use std::iter::FromIterator;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Default, Clone, Debug)]
pub struct Binding(HashMap<String, Option<Value>>);

impl FromIterator<String> for Binding {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self(iter.into_iter().map(|ident| (ident, None)).collect())
    }
}
