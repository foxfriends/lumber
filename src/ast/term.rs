use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) enum Term {
    Expression(Expression),
    PrefixOp(Operator, Box<Term>),
    InfixOp(Box<Term>, Operator, Box<Term>),
    Value(Pattern),
    #[cfg(feature = "builtin-sets")]
    SetAggregation(Pattern, Body),
    ListAggregation(Pattern, Body),
}

impl Term {
    pub fn prefix_operator(term: Term, operator: Operator) -> Term {
        Term::PrefixOp(operator, Box::new(term))
    }

    pub fn infix_operator(lhs: Term, operator: Operator, rhs: Term) -> Term {
        Term::InfixOp(Box::new(lhs), operator, Box::new(rhs))
    }

    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::term, pair.as_rule());
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::pattern => Some(Self::new_value(pair, context)),
            Rule::aggregation => Self::new_aggregation(pair, context),
            Rule::expression => Some(Self::Expression(Expression::new(pair, context)?)),
            _ => unreachable!(),
        }
    }

    fn new_value(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(Rule::pattern, pair.as_rule());
        Self::Value(Pattern::new(pair, context))
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
            Self::Expression(expression) => expression.handles_mut(),
            Self::Value(..) => Box::new(std::iter::empty()),
            Self::InfixOp(lhs, operator, rhs) => Box::new(
                std::iter::once(operator.handle_mut())
                    .chain(lhs.handles_mut())
                    .chain(rhs.handles_mut()),
            ),
            Self::PrefixOp(operator, term) => {
                Box::new(std::iter::once(operator.handle_mut()).chain(term.handles_mut()))
            }
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(.., body) => Box::new(body.handles_mut()),
            Self::ListAggregation(.., body) => Box::new(body.handles_mut()),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Expression(expression) => expression.identifiers(),
            Self::Value(pattern) => pattern.identifiers(),
            Self::PrefixOp(.., term) => term.identifiers(),
            Self::InfixOp(lhs, .., rhs) => Box::new(lhs.identifiers().chain(rhs.identifiers())),
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

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Expression(expression) => write!(f, "({})", expression),
            Self::Value(pattern) => pattern.fmt(f),
            Self::InfixOp(lhs, operator, rhs) => write!(f, "{} {} {}", lhs, operator, rhs),
            Self::PrefixOp(operator, term) => write!(f, "{} {}", operator, term),
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(..) => todo!(),
            Self::ListAggregation(pattern, body) => write!(f, "[{} : {}]", pattern, body),
        }
    }
}

impl From<Pattern> for Term {
    fn from(pattern: Pattern) -> Self {
        Self::Value(pattern)
    }
}
