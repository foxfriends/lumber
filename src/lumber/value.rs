#[cfg(feature = "builtin-sets")]
use super::Set;
use super::{List, Struct};
use crate::ast::{Literal, Pattern};
use ramp::{int::Int, rational::Rational};
use std::fmt::{self, Display, Formatter};

/// Basic untyped values as understood by Lumber.
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    /// An arbitrary size integer value.
    Integer(Int),
    /// An arbitrary precision rational value.
    Rational(Rational),
    /// A string value.
    String(String),
    /// An unordered, duplicate-free collection of values.
    #[cfg(feature = "builtin-sets")]
    Set(Set),
    /// An ordered collection of values, which may contain duplicates.
    List(List),
    /// A structural value. Atoms are really just structs with no fields.
    Struct(Struct),
}

impl Value {
    /// Constructs an integer value.
    pub fn integer(int: impl Into<Int>) -> Self {
        Self::Integer(int.into())
    }

    /// Constructs a rational value.
    pub fn rational(rat: impl Into<Rational>) -> Self {
        Self::Rational(rat.into())
    }

    /// Constructs a string value.
    pub fn string(string: impl Into<String>) -> Self {
        Self::String(string.into())
    }

    /// Constructs an atom value.
    pub fn atom(name: impl Into<String>) -> Self {
        Self::Struct(Struct::atom(name))
    }
}

impl From<Int> for Value {
    fn from(int: Int) -> Self {
        Self::Integer(int)
    }
}

impl From<Rational> for Value {
    fn from(rat: Rational) -> Self {
        Self::Rational(rat)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl<V> From<Vec<V>> for Value
where
    Value: From<V>,
{
    fn from(values: Vec<V>) -> Self {
        Self::List(values.into_iter().collect())
    }
}

impl From<Pattern> for Option<Value> {
    fn from(pattern: Pattern) -> Self {
        match pattern {
            Pattern::Variable(..) => None,
            Pattern::Wildcard => None,
            Pattern::Literal(Literal::Integer(int)) => Some(Value::Integer(int.to_owned())),
            Pattern::Literal(Literal::Rational(rat)) => Some(Value::Rational(rat.to_owned())),
            Pattern::Literal(Literal::String(string)) => Some(Value::String(string.to_owned())),
            Pattern::List(patterns, rest) => {
                let values = patterns.into_iter().map(Into::into).collect();
                let complete = rest.is_none();
                Some(Value::List(List::new(values, complete)))
            }
            #[cfg(feature = "builtin-sets")]
            Pattern::Set(patterns, rest) => {
                let values = patterns.into_iter().map(Into::into).collect();
                let complete = rest.is_none();
                Some(Value::Set(Set::new(values, complete)))
            }
            Pattern::Struct(structure) => {
                let values = structure.fields.into_iter().map(Into::into).collect();
                Some(Value::Struct(Struct::new(
                    structure.name.clone(),
                    &structure.arity,
                    values,
                )))
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Integer(int) => int.fmt(f),
            Value::Rational(rat) => rat.fmt(f),
            Value::String(string) => string.fmt(f),
            #[cfg(feature = "builtin-sets")]
            Value::Set(set) => set.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Struct(structure) => structure.fmt(f),
        }
    }
}
