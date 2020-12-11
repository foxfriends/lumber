use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) enum Term {
    Expression(Expression),
    Value(Pattern),
    #[cfg(feature = "builtin-sets")]
    SetAggregation(Pattern, Body),
    ListAggregation(Pattern, Body),
}

impl Term {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::term, pair.as_rule());
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::value => Self::new_value(pair, context),
            Rule::aggregation => Self::new_aggregation(pair, context),
            Rule::expression => Some(Self::Expression(Expression::new(pair, context)?)),
            _ => unreachable!(),
        }
    }

    fn new_value(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(Rule::value, pair.as_rule());
        let pair = just!(pair.into_inner());
        Some(Self::Value(Pattern::new_inner(pair, context)))
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
            #[cfg(feature = "builtin-sets")]
            Self::SetAggregation(.., body) => Box::new(body.handles_mut()),
            Self::ListAggregation(.., body) => Box::new(body.handles_mut()),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Expression(expression) => expression.identifiers(),
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

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Expression(expression) => write!(f, "({})", expression),
            Self::Value(pattern) => pattern.fmt(f),
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
