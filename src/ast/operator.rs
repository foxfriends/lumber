use super::*;
use crate::parser::Rule;
use ramp::Int;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Associativity {
    Left,
    Right,
}

impl Display for Associativity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum OpArity {
    Binary,
    Unary,
}

impl Display for OpArity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Binary => write!(f, "binary"),
            Self::Unary => write!(f, "unary"),
        }
    }
}

#[derive(Clone, Eq)]
pub(crate) enum Operator {
    Relation(Atom, OpArity),
    Expression(Atom, OpArity, Associativity, usize),
}

impl PartialEq for Operator {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Relation(lhs, larity), Self::Relation(rhs, rarity)) => {
                lhs == rhs && larity == rarity
            }
            (Self::Expression(lhs, larity, lass, ..), Self::Expression(rhs, rarity, rass, ..)) => {
                lhs == rhs && larity == rarity && lass == rass
            }
            _ => false,
        }
    }
}

impl Hash for Operator {
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self {
            Self::Relation(name, arity) => (name, arity, Option::<Associativity>::None).hash(h),
            Self::Expression(name, arity, assoc, ..) => (name, arity, Some(assoc)).hash(h),
        }
    }
}

impl Operator {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<(Self, Handle)> {
        assert_eq!(Rule::op, pair.as_rule());
        let mut pairs = pair.into_inner();
        let handle = Handle::new(pairs.next().unwrap(), context);
        let name = Atom::from(pairs.next().unwrap().to_string());
        match pairs.next() {
            Some(pair) => {
                let associativity = match pair.as_rule() {
                    Rule::left => Associativity::Left,
                    Rule::right => Associativity::Right,
                    _ => unreachable!(),
                };
                let level = Int::from_str_radix(pairs.next().unwrap().as_str(), 10).unwrap();
                if level >= 10 {
                    context.error_operator_precedence(name, level);
                    return None;
                }
                let len = handle.arity.len();
                if len == 2 {
                    Some((
                        Self::Expression(name, OpArity::Unary, associativity, usize::from(&level)),
                        handle,
                    ))
                } else if len == 3 {
                    Some((
                        Self::Expression(name, OpArity::Binary, associativity, usize::from(&level)),
                        handle,
                    ))
                } else {
                    context.error_operator_arity_expression(name, len);
                    None
                }
            }
            None => {
                let len = handle.arity.len();
                if len == 1 {
                    Some((Self::Relation(name, OpArity::Unary), handle))
                } else if len == 2 {
                    Some((Self::Relation(name, OpArity::Binary), handle))
                } else {
                    context.error_operator_arity_relation(name, len);
                    None
                }
            }
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Relation(atom, ..) | Self::Expression(atom, ..) => write!(f, "{}", atom),
        }
    }
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Relation(atom, ..) => write!(f, "{}", atom),
            Self::Expression(atom, arity, assoc, level) => {
                write!(f, "{} ({} {} {})", atom, arity, assoc, level)
            }
        }
    }
}
