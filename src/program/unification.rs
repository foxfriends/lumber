use super::*;

/// A unification against the database, used to build up a rule.
#[derive(Clone, Debug)]
pub enum Unification {
    /// A single query to be unified with the database.
    Query(Query),
    /// An entire sub-rule of unifications to be made.
    Body(Body),
}
