#[cfg(feature = "builtin-sets")]
use super::Set;
use super::{r#struct::Field, List, Struct};
use crate::ast::{Atom, Literal, Pattern};
use ramp::{int::Int, rational::Rational};
use std::any::Any;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

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
    #[cfg(feature = "builtin-sets")]
    Set(Set),
    /// An ordered collection of values, which may contain duplicates.
    List(List),
    /// A structural value. Atoms are really just structs with no fields.
    Struct(Struct),
    /// An unknown Rust value.
    Any(Rc<Box<dyn Any>>),
}

impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(lhs), Value::Integer(rhs)) => lhs == rhs,
            (Value::Rational(lhs), Value::Rational(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            #[cfg(feature = "builtin-sets")]
            (Value::Set(lhs), Value::Set(rhs)) => lhs == rhs,
            (Value::List(lhs), Value::List(rhs)) => lhs == rhs,
            (Value::Struct(lhs), Value::Struct(rhs)) => lhs == rhs,
            (Value::Any(lhs), Value::Any(rhs)) => Rc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
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

    /// Constructs a Lumber value containing an unknown Rust value.
    pub fn any(any: impl Any) -> Self {
        Self::Any(Rc::new(Box::new(any)))
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

impl From<&str> for Value {
    fn from(string: &str) -> Self {
        Self::String(string.to_owned())
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
            Pattern::Any(any) => Some(Value::Any(any)),
        }
    }
}

impl Into<Pattern> for Option<Value> {
    fn into(self) -> Pattern {
        match self {
            None => Pattern::Wildcard,
            Some(Value::Integer(int)) => Pattern::Literal(Literal::Integer(int)),
            Some(Value::Rational(rat)) => Pattern::Literal(Literal::Rational(rat)),
            Some(Value::String(string)) => Pattern::Literal(Literal::String(string)),
            Some(Value::List(List { values, complete })) => Pattern::List(
                values.into_iter().map(Into::into).collect(),
                if complete {
                    None
                } else {
                    Some(Box::new(Pattern::Wildcard))
                },
            ),
            #[cfg(feature = "builtin-sets")]
            Some(Value::Set(..)) => todo!(),
            Some(Value::Struct(Struct { name, fields })) => {
                let (arity, fields) = fields.into_iter().fold(
                    (vec![], vec![]),
                    |(mut arity, mut fields), (field, value)| {
                        match field {
                            Field::Index(..) => {
                                if let Some(crate::ast::Arity::Len(len)) = arity.last_mut() {
                                    *len += 1;
                                } else {
                                    arity.push(crate::ast::Arity::Len(1));
                                }
                            }
                            Field::Name(name) => {
                                arity.push(crate::ast::Arity::Name(Atom::from(name)));
                            }
                        }
                        fields.push(value.into());
                        (arity, fields)
                    },
                );
                Pattern::Struct(crate::ast::Struct {
                    name,
                    arity,
                    fields,
                })
            }
            Some(Value::Any(any)) => Pattern::Any(any),
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
            Value::Any(any) => write!(f, "[{:?}]", Rc::as_ptr(any)),
        }
    }
}
