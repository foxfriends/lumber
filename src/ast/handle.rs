use super::*;

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Arity {
    Len(usize),
    Name(Atom),
}

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Handle {
    /// The path and name of the predicate or function being described
    scope: Scope,
    /// The arity of the predicate or function being described
    arity: Vec<Arity>,
}
