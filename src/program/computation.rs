use super::*;
use crate::parser::Rule;
use pest::prec_climber::Assoc;
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

const fn left(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Left)
}

const fn right(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Right)
}

fn operation(
    pair: crate::Pair,
    context: &mut Context,
    output: Pattern,
) -> Option<Vec<Unification>> {
    assert_eq!(Rule::operation, pair.as_rule());
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
    let (mut work, result) = prec_climber.climb(
        pair.into_inner(),
        |pair| expression(pair, *context.borrow_mut()),
        |lhs, op, rhs| {
            let context = &mut *context.borrow_mut();
            let (mut lwork, lhs) = lhs?;
            let (mut rwork, rhs) = rhs?;
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
            Some((lwork, output))
        },
    )?;
    work.push(builtin::unify(output, result, context.into_inner()));
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
