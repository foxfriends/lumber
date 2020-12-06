use super::*;
use crate::parser::Rule;
use pest::prec_climber::Assoc;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};

const fn left(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Left)
}

#[allow(dead_code)]
const fn right(token: &'static str) -> Operator {
    Operator::new(token, Assoc::Right)
}

#[derive(Clone, Debug)]
pub(crate) enum Expression {
    Operation(Pattern, Vec<Unification>),
    Value(Pattern),
    #[cfg(feature = "builtin-sets")]
    SetAggregation(Pattern, Body),
    ListAggregation(Pattern, Body),
}

impl Expression {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::expression, pair.as_rule());
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::operation => Self::new_operation(pair, context),
            Rule::value => Self::new_value(pair, context),
            Rule::aggregation => Self::new_aggregation(pair, context),
            _ => unreachable!(),
        }
    }

    pub fn new_operation(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::operation, pair.as_rule());
        let (result, work) = operation(pair, context)?;
        Some(Self::Operation(result, work))
    }

    fn new_value(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::value, pair.as_rule());
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::call => {
                let output = Pattern::Variable(context.fresh_variable());
                Some(Self::Operation(
                    output.clone(),
                    vec![Unification::Query(Query::from_call(pair, context, output)?)],
                ))
            }
            _ => Some(Self::Value(Pattern::new_inner(pair, context))),
        }
    }

    fn new_aggregation(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::aggregation, pair.as_rule());
        let pair = just!(pair.into_inner());
        let constructor = match pair.as_rule() {
            #[cfg(feature = "builtin-sets")]
            Rule::set_aggregation => Self::SetAggregation,
            #[cfg(not(feature = "builtin-sets"))]
            Rule::set_aggregation => unimplemented!(
                "builtin-sets is not enabled, so set aggregation syntax cannot be used"
            ),
            Rule::list_aggregation => Self::ListAggregation,
            _ => unreachable!(),
        };
        let pair = just!(Rule::aggregation_body, pair.into_inner());
        let mut pairs = pair.into_inner();
        let output = Pattern::new(pairs.next().unwrap(), context);
        let body = Body::new_inner(pairs.next().unwrap(), context)?;
        Some(constructor(output, body))
    }

    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Operation(.., unifications) => {
                Box::new(unifications.iter_mut().flat_map(Unification::handles_mut))
            }
            Self::Value(..) => Box::new(std::iter::empty()),
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(.., body) => Box::new(body.handles_mut()),
            Self::ListAggregation(.., body) => Box::new(body.handles_mut()),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Operation(pattern, steps) => Box::new(
                pattern
                    .identifiers()
                    .chain(steps.iter().flat_map(|step| step.identifiers())),
            ),
            Self::Value(pattern) => pattern.identifiers(),
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(pattern, body) => {
                Box::new(pattern.identifiers().chain(body.identifiers()))
            }
            Self::ListAggregation(pattern, body) => {
                Box::new(pattern.identifiers().chain(body.identifiers()))
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Operation(.., input) => {
                for (i, unification) in input.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    unification.fmt(f)?;
                }
                Ok(())
            }
            Self::Value(pattern) => pattern.fmt(f),
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(..) => todo!(),
            Self::ListAggregation(pattern, body) => write!(f, "[{} : {}]", pattern, body),
        }
    }
}

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
