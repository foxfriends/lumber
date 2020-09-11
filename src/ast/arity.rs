use super::*;

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Arity {
    Len(usize),
    Name(Atom),
}
