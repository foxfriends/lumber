use super::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Query(Query),
    Variable(Identifier),
    Literal(Literal),
    List(Vec<Pattern>),
    Wildcard,
}
