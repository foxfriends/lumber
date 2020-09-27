use super::*;
use crate::parser::Rule;
use pest::prec_climber::Assoc;
use std::cell::RefCell;

const fn left(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Left)
}

const fn right(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Right)
}

#[derive(Clone, Debug)]
pub enum Computation {
    Operation(Pattern, Vec<Unification>),
    SetAggregation(Pattern, Body),
    ListAggregation(Pattern, Body),
}

impl Computation {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::computation, pair.as_rule());
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::operation => Self::new_operation(pair, context),
            Rule::aggregation => Self::new_aggregation(pair, context),
            _ => unreachable!(),
        }
    }

    fn new_operation(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::operation, pair.as_rule());
        let (result, work) = operation(pair, context)?;
        Some(Self::Operation(result, work))
    }

    fn new_aggregation(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        let pair = just!(pair.into_inner());
        let constructor = match pair.as_rule() {
            Rule::set_aggregation => Self::SetAggregation,
            Rule::list_aggregation => Self::ListAggregation,
            _ => unreachable!(),
        };
        let pair = just!(Rule::aggregation_body, pair.into_inner());
        let mut pairs = pair.into_inner();
        let output = Pattern::new(pairs.next().unwrap(), context);
        let body = Body::new_inner(pairs.next().unwrap(), context)?;
        Some(constructor(output, body))
    }

    pub(crate) fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Operation(.., unifications) => {
                Box::new(unifications.iter_mut().flat_map(Unification::handles_mut))
            }
            Self::SetAggregation(.., body) | Self::ListAggregation(.., body) => {
                Box::new(body.handles_mut())
            }
        }
    }

    pub(crate) fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Operation(pattern, steps) => Box::new(
                pattern
                    .identifiers()
                    .chain(steps.iter().flat_map(|step| step.identifiers())),
            ),
            Self::SetAggregation(pattern, body) | Self::ListAggregation(pattern, body) => {
                Box::new(pattern.identifiers().chain(body.identifiers()))
            }
        }
    }
}

fn expression(pair: crate::Pair, context: &mut Context) -> Option<(Pattern, Vec<Unification>)> {
    assert_eq!(Rule::expression, pair.as_rule());
    let pair = just!(pair.into_inner());
    match pair.as_rule() {
        Rule::value => {
            let pair = just!(pair.into_inner());
            match pair.as_rule() {
                Rule::call => {
                    let output = Pattern::Variable(context.fresh_variable());
                    let query = Query::from_call(pair, context, output.clone())?;
                    Some((output, vec![Unification::Query(query)]))
                }
                _ => Some((Pattern::new_inner(pair, context), vec![])),
            }
        }
        Rule::operation => operation(pair, context),
        _ => unreachable!(),
    }
}

fn operation(pair: crate::Pair, context: &mut Context) -> Option<(Pattern, Vec<Unification>)> {
    let prec_climber = PrecClimber::new(vec![
        left("||"),
        left("&&"),
        left("|"),
        left("^"),
        left("&"),
        left("==") | left("!="),
        left("<") | left(">") | left("<=") | left(">="),
        left("+") | left("-"),
        left("*") | left("/") | left("%"),
        right("**"),
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
                "+" => builtin::add(lhs, rhs, output.clone(), context),
                "-" => builtin::sub(lhs, rhs, output.clone(), context),
                "*" => builtin::mul(lhs, rhs, output.clone(), context),
                "/" => builtin::div(lhs, rhs, output.clone(), context),
                "%" => builtin::rem(lhs, rhs, output.clone(), context),
                "**" => builtin::exp(lhs, rhs, output.clone(), context),
                "==" => builtin::eq(lhs, rhs, output.clone(), context),
                "!=" => builtin::neq(lhs, rhs, output.clone(), context),
                "<" => builtin::lt(lhs, rhs, output.clone(), context),
                ">" => builtin::gt(lhs, rhs, output.clone(), context),
                "<=" => builtin::leq(lhs, rhs, output.clone(), context),
                ">=" => builtin::geq(lhs, rhs, output.clone(), context),
                "||" => builtin::or(lhs, rhs, output.clone(), context),
                "&&" => builtin::and(lhs, rhs, output.clone(), context),
                "|" => builtin::bitor(lhs, rhs, output.clone(), context),
                "&" => builtin::bitand(lhs, rhs, output.clone(), context),
                "^" => builtin::bitxor(lhs, rhs, output.clone(), context),
                token => match op.into_inner().next() {
                    Some(pair) => {
                        let scope = Scope::new(pair, context)?;
                        // TODO: can we check that `scope` is in scope here?
                        Unification::Query(Query::new(
                            Handle::binop(scope),
                            vec![lhs, rhs, output.clone()],
                        ))
                    }
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
