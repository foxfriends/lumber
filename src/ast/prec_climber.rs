//! [`PrecClimber`] implementation stolen and adapted from [`pest::prec_climber::PrecClimber`][]
//! so that it will can easily support user-defined operators, given that the operators are
//! declared before use.
use pest::{iterators::Pair, prec_climber::Assoc, RuleType};
use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::BitOr;

// use pest::prec_climber::Assoc;
// use std::cell::RefCell;
// const fn left(token: &'static str) -> Operator {
//     Operator::new(token, Assoc::Left)
// }

// #[allow(dead_code)]
// const fn right(token: &'static str) -> Operator {
//     Operator::new(token, Assoc::Right)
// }

#[derive(Debug)]
pub(crate) struct Operator<'s> {
    symbol: &'s str,
    assoc: Assoc,
    next: Option<Box<Operator<'s>>>,
}

impl<'s> Operator<'s> {
    pub const fn new(symbol: &'s str, assoc: Assoc) -> Self {
        Operator {
            symbol,
            assoc,
            next: None,
        }
    }
}

impl<'s> BitOr for Operator<'s> {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        fn assign_next<'s>(op: &mut Operator<'s>, next: Operator<'s>) {
            if let Some(ref mut child) = op.next {
                assign_next(child, next);
            } else {
                op.next = Some(Box::new(next));
            }
        }

        assign_next(&mut self, rhs);
        self
    }
}

#[derive(Debug)]
pub(crate) struct PrecClimber<'s> {
    ops: HashMap<&'s str, (u32, Assoc)>,
}

impl<'s> PrecClimber<'s> {
    pub fn new(ops: Vec<Operator<'s>>) -> PrecClimber<'s> {
        let ops = ops
            .into_iter()
            .zip(1..)
            .fold(HashMap::new(), |mut map, (op, prec)| {
                let mut next = Some(op);

                while let Some(op) = next.take() {
                    let Operator {
                        symbol,
                        assoc,
                        next: op_next,
                    } = op;

                    map.insert(symbol, (prec, assoc));
                    next = op_next.map(|op| *op);
                }

                map
            });

        PrecClimber { ops }
    }

    pub fn climb<'i, R, P, F, G, T>(&self, mut pairs: P, mut primary: F, mut infix: G) -> T
    where
        R: RuleType,
        P: Iterator<Item = Pair<'i, R>>,
        F: FnMut(Pair<'i, R>) -> T,
        G: FnMut(T, Pair<'i, R>, T) -> T,
    {
        let lhs = primary(
            pairs
                .next()
                .expect("precedence climbing requires a non-empty Pairs"),
        );
        self.climb_rec(lhs, 0, &mut pairs.peekable(), &mut primary, &mut infix)
    }

    fn climb_rec<'i, R, P, F, G, T>(
        &self,
        mut lhs: T,
        min_prec: u32,
        pairs: &mut Peekable<P>,
        primary: &mut F,
        infix: &mut G,
    ) -> T
    where
        R: RuleType,
        P: Iterator<Item = Pair<'i, R>>,
        F: FnMut(Pair<'i, R>) -> T,
        G: FnMut(T, Pair<'i, R>, T) -> T,
    {
        while pairs.peek().is_some() {
            let token = pairs.peek().unwrap().as_str();
            let &(prec, _) = self
                .ops
                .get(&token)
                .unwrap_or(&(u32::max_value(), Assoc::Left));
            if prec >= min_prec {
                let op = pairs.next().unwrap();
                let mut rhs = primary(pairs.next().expect(
                    "infix operator must be followed by \
                     a primary expression",
                ));

                while pairs.peek().is_some() {
                    let token = pairs.peek().unwrap().as_str();
                    let &(new_prec, assoc) = self
                        .ops
                        .get(&token)
                        .unwrap_or(&(u32::max_value(), Assoc::Left));
                    if new_prec > prec || assoc == Assoc::Right && new_prec == prec {
                        rhs = self.climb_rec(rhs, new_prec, pairs, primary, infix);
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
}


/*
fn expression(pair: crate::Pair, context: &mut Context) -> Option<(Pattern, Vec<Unification>)> {
    let expression = Expression::new(pair, context)?;
    match expression {
        Expression::Value(value) => Some((value, vec![])),
        Expression::Operation(output, steps) => Some((output, steps)),
        aggregation => {
            let output = Pattern::Variable(context.fresh_variable());
            Some((
                output.clone(),
                vec![Unification::Assumption(output, aggregation)],
            ))
        }
    }
}

fn operation(pair: crate::Pair, context: &mut Context) -> Option<(Pattern, Vec<Unification>)> {
    let prec_climber = PrecClimber::new(vec![
        // left("\\"),
        // left("||"),
        // left("&&"),
        left("|"),
        left("^"),
        left("&"),
        // left("==") | left("!="),
        // left("<") | left(">") | left("<=") | left(">="),
        left("+") | left("-"),
        left("*") | left("/") | left("%"),
        // right("**")
    ]);
    let context = RefCell::new(context);
    prec_climber.climb(
        pair.into_inner(),
        |pair| expression(pair, *context.borrow_mut()),
        |lhs, op, rhs| {
            let context = &mut *context.borrow_mut();
            let (lhs, mut lwork) = lhs?;
            let (rhs, mut rwork) = rhs?;
            let output = Pattern::Variable(context.fresh_variable());
            let operation = match op.as_str() {
                "+" => builtin::add(lhs, rhs, output.clone()),
                "-" => builtin::sub(lhs, rhs, output.clone()),
                "*" => builtin::mul(lhs, rhs, output.clone()),
                "/" => builtin::div(lhs, rhs, output.clone()),
                "%" => builtin::rem(lhs, rhs, output.clone()),
                "|" => builtin::bitor(lhs, rhs, output.clone()),
                "&" => builtin::bitand(lhs, rhs, output.clone()),
                "^" => builtin::bitxor(lhs, rhs, output.clone()),
                token => match op.into_inner().next() {
                    Some(pair) => match pair.as_rule() {
                        Rule::named_operator => {
                            let pair = just!(pair.into_inner());
                            let scope = Scope::new(pair, context)?;
                            Unification::Query(Query::new(
                                Handle::binop(scope),
                                vec![lhs, rhs, output.clone()],
                            ))
                        }
                        Rule::symbolic_operator => {
                            context.error_unrecognized_operator(token);
                            return None;
                        }
                        _ => unreachable!(),
                    },
                    None => {
                        context.error_unrecognized_operator(token);
                        return None;
                    }
                },
            };
            lwork.append(&mut rwork);
            lwork.push(operation);
            Some((output, lwork))
        },
    )
}
*/
