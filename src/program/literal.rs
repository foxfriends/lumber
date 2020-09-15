use ramp::{int::Int, rational::Rational};

/// A literal value, which cannot be further pattern matched.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Literal {
    /// A single integer.
    Integer(Int),
    /// A fractional number.
    Decimal(Rational),
    /// A string.
    String(String),
}
