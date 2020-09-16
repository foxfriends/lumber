use super::*;
use crate::parser::Rule;
use pest::prec_climber::Assoc::*;
use std::cell::RefCell;

fn expression(pair: crate::Pair, context: &mut Context) -> Option<(Vec<Unification>, Pattern)> {
    assert_eq!(Rule::expression, pair.as_rule());
    let pair = just!(pair.into_inner());
    match pair.as_rule() {
        Rule::value => {
            let pair = just!(pair.into_inner());
            match pair.as_rule() {
                Rule::call => {
                    let output = Pattern::Variable(context.fresh_variable());
                    let query = Query::from_call(pair, context, output.clone())?;
                    Some((vec![Unification::Query(query)], output))
                }
                _ => Some((vec![], Pattern::new_inner(pair, context))),
            }
        }
        Rule::operation => {
            let output = Pattern::Variable(context.fresh_variable());
            Some((operation(pair, context, output.clone())?, output))
        }
        _ => unreachable!(),
    }
}

fn operation(
    pair: crate::Pair,
    context: &mut Context,
    output: Pattern,
) -> Option<Vec<Unification>> {
    assert_eq!(Rule::operation, pair.as_rule());
    let prec_climber = PrecClimber::new(vec![
        Operator::new("+", Left) | Operator::new("-", Left),
        Operator::new("*", Left) | Operator::new("/", Left) | Operator::new("%", Left),
    ]);

    let context = RefCell::new(context);
    let (mut work, result) = prec_climber.climb(
        pair.into_inner(),
        |pair| expression(pair, *context.borrow_mut()),
        |lhs, op, rhs| {
            let (mut lwork, lhs) = lhs?;
            let (mut rwork, rhs) = rhs?;
            let output = Pattern::Variable(context.borrow_mut().fresh_variable());
            let operation = match op.as_str() {
                "+" => builtin::add(lhs, rhs, output.clone()),
                "-" => builtin::sub(lhs, rhs, output.clone()),
                "*" => builtin::mul(lhs, rhs, output.clone()),
                "/" => builtin::div(lhs, rhs, output.clone()),
                "%" => builtin::rem(lhs, rhs, output.clone()),
                _ => todo!(),
            };
            lwork.append(&mut rwork);
            lwork.push(operation);
            Some((lwork, output))
        },
    )?;
    work.push(builtin::unify(output, result));
    Some(work)
}

pub(crate) fn computation(
    pair: crate::Pair,
    context: &mut Context,
    output: Pattern,
) -> Option<Vec<Unification>> {
    assert_eq!(Rule::computation, pair.as_rule());
    operation(just!(Rule::operation, pair.into_inner()), context, output)
}
