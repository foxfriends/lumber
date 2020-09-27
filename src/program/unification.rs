use super::*;
use crate::parser::Rule;

/// A unification against the database, used to build up a rule.
#[derive(Clone, Debug)]
pub enum Unification {
    /// A single query to be unified with the database.
    Query(Query),
    /// An entire sub-rule of unifications to be made.
    Body(Body),
    /// An assumption, where a pattern assumes a value.
    Assumption(Pattern, Computation),
}

impl Unification {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::unification);
        let pair = just!(pair.into_inner());
        let unification = match pair.as_rule() {
            Rule::assumption => Self::from_assumption(pair, context)?,
            Rule::predicate => Self::Query(Query::from_predicate(pair, context)?),
            Rule::procession => Self::Body(Body::new_inner(pair, context)?),
            _ => unreachable!(),
        };
        Some(unification)
    }

    pub(crate) fn from_assumption(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::assumption);
        let mut pairs = pair.into_inner();
        let output = Pattern::new(pairs.next().unwrap(), context);
        Some(Self::Assumption(
            output,
            Computation::new(pairs.next().unwrap(), context)?,
        ))
    }

    pub(crate) fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Query(query) => Box::new(std::iter::once(query.as_mut())),
            Self::Body(body) => Box::new(body.handles_mut()),
            Self::Assumption(_, computation) => computation.handles_mut(),
        }
    }

    pub(crate) fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Query(query) => Box::new(query.identifiers()),
            Self::Body(body) => Box::new(body.identifiers()),
            Self::Assumption(pattern, computation) => {
                Box::new(pattern.identifiers().chain(computation.identifiers()))
            }
        }
    }
}
