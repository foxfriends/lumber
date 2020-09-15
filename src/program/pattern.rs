use super::*;
use crate::parser::Rule;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Debug)]
pub enum Pattern {
    /// A structured pattern (unifies structurally with another query of the same name).
    Struct(Struct),
    /// A single variable (unifies with anything but only once).
    Variable(Identifier),
    /// A literal value (unifies only with itself).
    Literal(Literal),
    /// A list of patterns (unifies with a list of the same length where the paterns each unify).
    List(Vec<Pattern>),
    /// A wildcard (unifies with anything).
    Wildcard,
}

impl Pattern {
    pub(crate) fn new<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Self {
        assert_eq!(pair.as_rule(), Rule::pattern);
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::struct_ => todo!(),
            Rule::literal => todo!(),
            Rule::variable => todo!(),
            Rule::list => todo!(),
            Rule::wildcard => Self::Wildcard,
            _ => unreachable!(),
        }
    }
}
