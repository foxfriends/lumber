use super::*;

/// A handle to a predicate.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Handle {
    /// The path and name of the predicate or function being described
    scope: Scope,
    /// The arity of the predicate or function being described
    arity: Vec<Arity>,
}
