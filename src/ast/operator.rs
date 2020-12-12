use super::*;
use crate::parser::Rule;
use ramp::Int;
use std::fmt::{self, Debug, Display, Formatter};

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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum OpKey {
    Relation(Atom, OpArity),
    Expression(Atom, OpArity),
}

#[derive(Clone)]
pub(crate) struct Operator {
    key: OpKey,
    handle: Handle,
    assoc: Associativity,
    prec: usize,
}

impl Operator {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::op, pair.as_rule());
        let mut pairs = pair.into_inner();
        let handle = Handle::new(pairs.next().unwrap(), context);
        let name = Atom::from(pairs.next().unwrap().to_string());
        match pairs.next() {
            Some(pair) => {
                let assoc = match pair.as_rule() {
                    Rule::left => Associativity::Left,
                    Rule::right => Associativity::Right,
                    _ => unreachable!(),
                };
                let level = Int::from_str_radix(pairs.next().unwrap().as_str(), 10).unwrap();
                if level >= 10 {
                    context.error_operator_precedence(name, level);
                    return None;
                }
                let key = match handle.arity.len() {
                    2 => OpKey::Expression(name, OpArity::Unary),
                    3 => OpKey::Expression(name, OpArity::Binary),
                    len => {
                        context.error_operator_arity_expression(name, len);
                        return None;
                    }
                };
                Some(Self {
                    key,
                    handle,
                    assoc,
                    prec: usize::from(&level),
                })
            }
            None => {
                let key = match handle.arity.len() {
                    1 => OpKey::Relation(name, OpArity::Unary),
                    2 => OpKey::Relation(name, OpArity::Binary),
                    len => {
                        context.error_operator_arity_expression(name, len);
                        return None;
                    }
                };
                Some(Self {
                    key,
                    handle,
                    assoc: Associativity::Right,
                    prec: 0,
                })
            }
        }
    }

    pub fn key(&self) -> OpKey {
        self.key.clone()
    }

    pub fn add_lib(&mut self, lib: Atom) {
        self.handle.add_lib(lib);
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Display for OpKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Relation(atom, ..) | Self::Expression(atom, ..) => write!(f, "{}", atom),
        }
    }
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self {
                key: OpKey::Relation(atom, arity),
                ..
            } => write!(f, "{} ({})", atom, arity),
            Self {
                key: OpKey::Expression(atom, arity),
                assoc,
                prec,
                ..
            } => {
                write!(f, "{} ({} {} {})", atom, arity, assoc, prec)
            }
        }
    }
}