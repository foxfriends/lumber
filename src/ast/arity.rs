use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// The arity portion of a predicate handle.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Arity {
    /// Number of consecutive unnamed arguments.
    Len(u32),
    /// A singled named argument.
    Name(Atom),
}

impl Arity {
    pub fn new(pair: crate::Pair) -> Self {
        assert_eq!(pair.as_rule(), Rule::arity);
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            // TODO: do we really want to panic on arities longer than 2^32?
            Rule::integer_10 => Self::Len(pair.as_str().parse().unwrap()),
            Rule::atom => Self::Name(Atom::new(pair)),
            _ => unreachable!(),
        }
    }

    pub fn can_alias(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Len(a), Self::Len(b)) => a == b,
            (Self::Name(..), Self::Name(..)) => true,
            _ => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&str>> {
        match self {
            Self::Len(len) => {
                Box::new((0..*len).map(|_| None)) as Box<dyn Iterator<Item = Option<&str>>>
            }
            Self::Name(name) => Box::new(std::iter::once(Some(name.as_ref()))),
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
