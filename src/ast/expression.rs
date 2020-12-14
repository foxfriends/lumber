use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) struct Expression(Vec<Op>);

#[derive(Clone, Debug)]
pub(crate) enum Op {
    RatorRef(Atom),
    Rator(Operator),
    Rand(Term),
}

impl Expression {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::expression, pair.as_rule());
        let operation = pair
            .into_inner()
            .map(|pair| match pair.as_rule() {
                Rule::operator => Some(Op::RatorRef(Atom::from(pair.as_str()))),
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
                .flat_map(|op| -> Box<dyn Iterator<Item = &mut Handle>> {
                    match op {
                        Op::RatorRef(..) => Box::new(std::iter::empty()),
                        Op::Rator(operator) => Box::new(std::iter::once(operator.handle_mut())),
                        Op::Rand(term) => term.handles_mut(),
                    }
                }),
        )
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        Box::new(
            self.0
                .iter()
                .filter_map(|op| match op {
                    Op::Rand(term) => Some(term),
                    _ => None,
                })
                .flat_map(|term| term.identifiers()),
        )
    }

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Handle>>(&mut self, mut resolve: F) {
        for _i in 0..self.0.len() {
            // TODO: what is the algorithm?
            // If left is Term and right is Term => infix
            // If left is Term and right is not => postfix or infix
            // If right is Term and left is not => prefix or infix
            // Within a chain of multiple operators, exactly one is infix
            // If there are multiple candidates for the infix operator, fix them in order of precedence
        }
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
            Op::RatorRef(name) => write!(f, "{}", name.as_ref()),
            Op::Rator(operator) => operator.fmt(f),
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
