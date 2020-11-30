#![allow(clippy::redundant_allocation)]
#[cfg(feature = "builtin-sets")]
use super::Set;
use super::{List, Record, Struct};
use crate::ast::{Atom, Literal, Pattern};
use ramp::{int::Int, rational::Rational};
use std::any::Any;
use std::collections::HashMap;
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
    /// A set of key value(s) pairs.
    Record(Record),
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
            (Value::Record(lhs), Value::Record(rhs)) => lhs == rhs,
            (Value::Any(lhs), Value::Any(rhs)) => Rc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

macro_rules! is_variant {
    ($name:ident, $variant:ident) => {
        /// Gets this value as a $ty.
        pub fn $name(&self) -> bool {
            matches!(self, Self::$variant(..))
        }
    };
}

macro_rules! as_variant {
    ($name:ident, $ty:ty, $variant:ident) => {
        /// Gets this value as a $ty.
        pub fn $name(&self) -> Option<&$ty> {
            match self {
                Self::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
}

macro_rules! as_variant_mut {
    ($name:ident, $ty:ty, $variant:ident) => {
        /// Gets this value as a $ty, mutably.
        pub fn $name(&mut self) -> Option<&mut $ty> {
            match self {
                Self::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
}

impl Value {
    /// Constructs an integer value.
    pub fn integer(int: impl Into<Int>) -> Self {
        Self::Integer(int.into())
    }

    is_variant!(is_integer, Integer);
    as_variant!(as_integer, Int, Integer);
    as_variant_mut!(as_integer_mut, Int, Integer);

    /// Constructs a rational value.
    pub fn rational(rat: impl Into<Rational>) -> Self {
        Self::Rational(rat.into())
    }

    is_variant!(is_rational, Rational);
    as_variant!(as_rational, Rational, Rational);
    as_variant_mut!(as_rational_mut, Rational, Rational);

    /// Constructs a string value.
    pub fn string(string: impl Into<String>) -> Self {
        Self::String(string.into())
    }

    is_variant!(is_string, String);
    as_variant!(as_string, String, String);
    as_variant_mut!(as_string_mut, String, String);

    /// Constructs an atom value.
    pub fn atom(name: impl Into<String>) -> Self {
        Self::Struct(Struct::atom(name))
    }

    is_variant!(is_struct, Struct);
    as_variant!(as_struct, Struct, Struct);
    as_variant_mut!(as_struct_mut, Struct, Struct);

    /// Constructs a Lumber value containing an unknown Rust value.
    pub fn any(any: impl Any) -> Self {
        Self::Any(Rc::new(Box::new(any)))
    }

    /// Constructs a Lumber value containing a list of other values.
    pub fn list<V>(values: impl IntoIterator<Item = V>) -> Self
    where
        Value: From<V>,
    {
        Self::List(values.into_iter().collect())
    }

    is_variant!(is_list, List);
    as_variant!(as_list, List, List);
    as_variant_mut!(as_list_mut, List, List);

    /// Constructs a Lumber value containing a record.
    pub fn record(values: HashMap<String, Option<Value>>) -> Self {
        Self::Record(Record::new(
            values
                .into_iter()
                .map(|(k, v)| (Atom::from(k), v))
                .collect(),
            true,
        ))
    }

    is_variant!(is_record, Record);
    as_variant!(as_record, Record, Record);
    as_variant_mut!(as_record_mut, Record, Record);

    /// Constructs a Lumber value by serializing a Rust value using Serde.
    #[cfg(feature = "serde")]
    pub fn serialize<T: serde::Serialize>(value: &T) -> crate::Result<Self> {
        crate::ser::to_value(value)
    }

    /// Deserializes a Lumber value to a Rust value using Serde.
    #[cfg(feature = "serde")]
    pub fn deserialize<'de, T>(&'de self) -> crate::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        crate::de::from_value(self)
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
            Pattern::Bound | Pattern::Unbound => None,
            Pattern::Literal(Literal::Integer(int)) => Some(Value::Integer(int)),
            Pattern::Literal(Literal::Rational(rat)) => Some(Value::Rational(rat)),
            Pattern::Literal(Literal::String(string)) => Some(Value::String(string)),
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
            Pattern::Record(fields, rest) => {
                let values = fields
                    .into_iter()
                    .map(|(key, pattern)| (key, pattern.into()))
                    .collect();
                let complete = rest.is_none();
                Some(Value::Record(Record::new(values, complete)))
            }
            Pattern::Struct(structure) => {
                let contents = structure
                    .contents
                    .map(|contents| Box::new((*contents).into()));
                Some(Value::Struct(Struct::raw(structure.name, contents)))
            }
            Pattern::Any(any) => Some(Value::Any(any)),
            Pattern::All(patterns) => patterns.into_iter().find_map(|pattern| pattern.into()),
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
            Some(Value::Record(Record { fields, complete })) => Pattern::Record(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
                if complete {
                    None
                } else {
                    Some(Box::new(Pattern::Wildcard))
                },
            ),
            Some(Value::Struct(Struct { name, contents })) => {
                let contents = contents.map(|contents| Box::new((*contents).into()));
                Pattern::Struct(crate::ast::Struct::from_parts(name, contents))
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
            Value::String(string) => write!(f, "{:?}", string),
            #[cfg(feature = "builtin-sets")]
            Value::Set(set) => set.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Record(record) => record.fmt(f),
            Value::Struct(structure) => structure.fmt(f),
            Value::Any(any) => write!(f, "[{:?}]", Rc::as_ptr(any)),
        }
    }
}
