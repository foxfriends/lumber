use super::*;
use crate::parser::Rule;
use ramp::Int;
use std::fmt::{self, Display, Formatter};

/// The arity portion of a predicate handle.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Arity {
    /// Number of consecutive unnamed arguments.
    Len(Int),
    /// A singled named argument.
    Name(Atom),
}

impl Arity {
    pub(crate) fn new<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Self {
        assert_eq!(pair.as_rule(), Rule::arity);
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::integer_10 => Self::Len(pair.as_str().parse().unwrap()),
            Rule::atom => Self::Name(context.atomizer.atomize(pair)),
            _ => unreachable!(),
        }
    }
}

impl Display for Arity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Arity::Len(len) => write!(f, "/{}", len),
            Arity::Name(atom) => write!(f, ":{}", atom),
        }
    }
}
