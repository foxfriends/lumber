use super::*;
use crate::parser::Rule;

/// A unification against the database, used to build up a rule.
#[derive(Clone, Debug)]
pub(crate) enum Unification {
    /// A single query to be unified with the database.
    Query(Query),
    /// An entire sub-rule of unifications to be made.
    Body(Body),
    /// An assumption, where a pattern assumes a value.
    Assumption(Pattern, Expression),
    /// A unification that always fails.
    Never,
}

impl Unification {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::unification);
        let pair = just!(pair.into_inner());
        let unification = match pair.as_rule() {
            Rule::assumption => Self::from_assumption(pair, context)?,
            Rule::predicate => Self::Query(Query::from_predicate(pair, context)?),
            Rule::disjunction => Self::Body(Body::new_inner(pair, context)?),
            Rule::never => Self::Never,
            _ => unreachable!(),
        };
        Some(unification)
    }

    pub fn from_assumption(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::assumption);
        let mut pairs = pair.into_inner();
        let output = Pattern::new(pairs.next().unwrap(), context);
        Some(Self::Assumption(
            output,
            Expression::new_operation(pairs.next().unwrap(), context)?,
        ))
    }

    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Query(query) => Box::new(std::iter::once(query.as_mut())),
            Self::Body(body) => Box::new(body.handles_mut()),
            Self::Assumption(_, expression) => expression.handles_mut(),
            Self::Never => Box::new(std::iter::empty()),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Query(query) => Box::new(query.identifiers()),
            Self::Body(body) => Box::new(body.identifiers()),
            Self::Assumption(pattern, expression) => {
                Box::new(pattern.identifiers().chain(expression.identifiers()))
            }
            Self::Never => Box::new(std::iter::empty()),
        }
    }
}
