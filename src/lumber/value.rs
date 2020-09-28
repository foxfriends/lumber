use super::{Set, Struct};
use ramp::{int::Int, rational::Rational};

/// Basic untyped values as understood by Lumber.
#[derive(Clone, Debug)]
pub enum Value {
    /// An arbitrary size integer value.
    Integer(Int),
    /// An arbitrary precision rational value.
    Rational(Rational),
    /// A string value.
    String(String),
    /// A meaningless atomic symbol.
    Atom(String),
    /// An unordered, duplicate-free collection of values.
    Set(Set),
    /// An ordered collection of values, which may contain duplicates.
    List(Vec<Option<Value>>),
    /// A structural value.
    Struct(Struct),
}
