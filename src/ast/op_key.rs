use super::*;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum OpKey {
    Relation(Atom, OpArity),
    Expression(Atom, OpArity),
}

impl OpKey {
    pub fn all_types(name: Atom) -> impl Iterator<Item = OpKey> {
        vec![
            OpKey::Relation(name.clone(), OpArity::Unary),
            OpKey::Relation(name.clone(), OpArity::Binary),
            OpKey::Expression(name.clone(), OpArity::Unary),
            OpKey::Expression(name, OpArity::Binary),
        ]
        .into_iter()
    }

    pub fn name(&self) -> Atom {
        match self {
            Self::Relation(name, ..) | Self::Expression(name, ..) => name.clone(),
        }
    }
}

impl Display for OpKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Relation(atom, ..) | Self::Expression(atom, ..) => write!(f, "{}", atom.as_ref()),
        }
    }
}
