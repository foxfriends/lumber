use super::*;
use ramp::{int::Int, rational::Rational};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Literal {
    Integer(Int),
    Decimal(Rational),
    String(String),
    Atom(Atom),
}
