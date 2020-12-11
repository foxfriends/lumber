use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) struct Expression(Vec<Op>);

#[derive(Clone, Debug)]
pub(crate) enum Op {
    Rator(String), // TODO: operators
    Rand(Term),
}

impl Expression {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::expression, pair.as_rule());
        let operation = pair
            .into_inner()
            .map(|pair| match pair.as_rule() {
                Rule::operator => Some(Op::Rator(pair.to_string())),
                Rule::term => Some(Op::Rand(Term::new(pair, context)?)),
                _ => unreachable!(),
            })
            .collect::<Option<_>>()?;
        Some(Self(operation))
    }

    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        Box::new(
            self.0
                .iter_mut()
                .filter_map(|op| match op {
                    Op::Rator(..) => None,
                    Op::Rand(term) => Some(term),
                })
                .flat_map(|term| term.handles_mut()),
        )
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        Box::new(
            self.0
                .iter()
                .filter_map(|op| match op {
                    Op::Rator(..) => None,
                    Op::Rand(term) => Some(term),
                })
                .flat_map(|term| term.identifiers()),
        )
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(" ")
            .fmt(f)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Op::Rator(rator) => rator.fmt(f),
            Op::Rand(rand) => rand.fmt(f),
        }
    }
}

impl<T> From<T> for Expression
where
    Term: From<T>,
{
    fn from(value: T) -> Self {
        Self(vec![Op::Rand(Term::from(value))])
    }
}
