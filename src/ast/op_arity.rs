use std::fmt::{self, Debug, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum OpArity {
    Binary,
    Unary,
}

impl Display for OpArity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Binary => write!(f, "binary"),
            Self::Unary => write!(f, "unary"),
        }
    }
}
