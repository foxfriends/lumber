use super::*;

/// The arity portion of a predicate handle.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Arity {
    /// Number of consecutive unnamed arguments.
    Len(usize),
    /// A singled named argument.
    Name(Atom),
}
