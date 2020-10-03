use super::{List, Set, Struct};
use crate::ast::{Literal, Pattern};
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
    /// An unordered, duplicate-free collection of values.
    Set(Set),
    /// An ordered collection of values, which may contain duplicates.
    List(List),
    /// A structural value. Atoms are really just structs with no fields.
    Struct(Struct),
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
