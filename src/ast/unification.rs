use super::*;

#[derive(Clone, Debug)]
pub enum Unification {
    Query(Query),
    Body(Body),
}
