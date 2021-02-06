use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) enum Op<O, T> {
    Rator(O),
    Rand(T),
}

impl<O, T> Display for Op<O, T>
where
    O: AsRef<str>,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Op::Rator(name) => write!(f, "{}", name.as_ref()),
            Op::Rand(rand) => rand.fmt(f),
        }
    }
}

impl<O, T> Op<O, T> {
    pub fn into_rator(self) -> Option<O> {
        match self {
            Self::Rator(o) => Some(o),
            _ => None,
        }
    }

    pub fn into_rand(self) -> Option<T> {
        match self {
            Self::Rand(t) => Some(t),
            _ => None,
        }
    }
}
