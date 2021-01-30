use super::*;
use crate::climb::*;
use crate::parser::Rule;
use ramp::Int;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Eq, PartialEq, Hash)]
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
        let name = Atom::from(pairs.next().unwrap().as_str());
        let handle = Handle::new(pairs.next().unwrap(), context);
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
                    2 => {
                        if assoc != Associativity::Right || level != 9 {
                            context.error_unary_operator_restriction(name);
                            return None;
                        }
                        OpKey::Expression(name, OpArity::Unary)
                    }
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
                        context.error_operator_arity_relation(name, len);
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

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }
}

impl Climbable for Operator {
    fn assoc(&self) -> Associativity {
        self.assoc
    }

    fn prec(&self) -> usize {
        self.prec
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self {
                key: OpKey::Relation(atom, arity),
                handle,
                ..
            } => write!(f, "{} ({} = {})", atom, arity, handle),
            Self {
                key: OpKey::Expression(atom, arity),
                assoc,
                prec,
                handle,
            } => {
                write!(f, "{} ({} {} {} = {})", atom, arity, assoc, prec, handle)
            }
        }
    }
}
