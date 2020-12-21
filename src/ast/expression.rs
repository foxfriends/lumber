use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) struct Expression(Vec<Op<Atom>>);

#[derive(Clone, Debug)]
pub(crate) enum Op<O = Atom, T = Term> {
    Rator(O),
    Rand(T),
}

impl<O, T> Op<O, T> {
    fn into_rator(self) -> Option<O> {
        match self {
            Self::Rator(o) => Some(o),
            _ => None,
        }
    }

    fn into_rand(self) -> Option<T> {
        match self {
            Self::Rand(t) => Some(t),
            _ => None,
        }
    }
}

impl Expression {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::expression, pair.as_rule());
        let operation = pair
            .into_inner()
            .map(|pair| match pair.as_rule() {
                Rule::operator => Some(Op::Rator(Atom::from(pair.as_str()))),
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
                        Op::Rator(..) => Box::new(std::iter::empty()),
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

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Operator>>(&mut self, resolve: F) {
        if let Some(term) = self.climb_operators(
            resolve,
            Clone::clone,
            Term::prefix_operator,
            Term::infix_operator,
        ) {
            self.0 = vec![Op::Rand(term)];
        }
    }

    pub fn climb_operators<
        Out,
        Res: FnMut(&OpKey) -> Option<Operator>,
        Init: Fn(&Term) -> Out,
        Prefix: Fn(Out, Operator) -> Out,
        Infix: Copy + Fn(Out, Operator, Out) -> Out,
    >(
        &self,
        mut resolve: Res,
        init: Init,
        prefix: Prefix,
        infix: Infix,
    ) -> Option<Out> {
        let mut collapsed = vec![];
        // Resolve unary operators
        let mut arity = OpArity::Unary;
        let mut prefixes = vec![];
        for op in &self.0 {
            match op {
                Op::Rator(name) if arity == OpArity::Unary => {
                    let operator = resolve(&OpKey::Expression(name.clone(), arity))?;
                    prefixes.push(operator);
                }
                Op::Rator(name) => {
                    let operator = resolve(&OpKey::Expression(name.clone(), arity))?;
                    arity = OpArity::Unary;
                    collapsed.push(Op::Rator(operator));
                }
                Op::Rand(term) => {
                    arity = OpArity::Binary;
                    let reduced = prefixes.drain(..).rev().fold(init(term), &prefix);
                    collapsed.push(Op::Rand(reduced));
                }
            }
        }

        Some(climb(collapsed.into_iter(), infix))
    }

    pub fn single_term(&self) -> &Term {
        assert_eq!(self.0.len(), 1);
        match self.0.first().unwrap() {
            Op::Rator(..) => unreachable!(),
            Op::Rand(rand) => rand,
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

impl<O> Display for Op<O>
where
    O: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Op::Rator(name) => write!(f, "{}", name.as_ref()),
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

fn climb<Out, Infix: Copy + Fn(Out, Operator, Out) -> Out>(
    mut inputs: impl Iterator<Item = Op<Operator, Out>>,
    infix: Infix,
) -> Out {
    let lhs = inputs.next().and_then(Op::into_rand).unwrap();
    climb_rec(lhs, 0, &mut inputs.peekable(), infix)
}

fn climb_rec<Out, P, Infix: Copy + Fn(Out, Operator, Out) -> Out>(
    mut lhs: Out,
    min_prec: usize,
    inputs: &mut std::iter::Peekable<P>,
    infix: Infix,
) -> Out
where
    P: Iterator<Item = Op<Operator, Out>>,
{
    while inputs.peek().is_some() {
        let item = inputs.peek().unwrap();
        let prec = match item {
            Op::Rator(rator) => rator.prec(),
            _ => unreachable!(),
        };
        if prec >= min_prec {
            let op = inputs.next().and_then(Op::into_rator).unwrap();
            let mut rhs = inputs.next().and_then(Op::into_rand).unwrap();

            while inputs.peek().is_some() {
                let item = inputs.peek().unwrap();
                let (new_prec, assoc) = match item {
                    Op::Rator(rator) => (rator.prec(), rator.assoc()),
                    _ => unreachable!(),
                };
                if new_prec > prec || assoc == Associativity::Right && new_prec == prec {
                    rhs = climb_rec(rhs, new_prec, inputs, infix);
                } else {
                    break;
                }
            }

            lhs = infix(lhs, op, rhs);
        } else {
            break;
        }
    }

    lhs
}
