use super::*;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Debug)]
pub enum Pattern {
    /// A query-shaped pattern (unifies structurally with another query of the same name).
    Query(Query),
    /// A single variable (unifies with anything but only once).
    Variable(Identifier),
    /// A literal value (unifies only with itself).
    Literal(Literal),
    /// A list of patterns (unifies with a list of the same length where the paterns each unify).
    List(Vec<Pattern>),
    /// A wildcard (unifies with anything).
    Wildcard,
}
