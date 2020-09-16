use super::*;
use crate::parser::Rule;

/// A unification against the database, used to build up a rule.
#[derive(Clone, Debug)]
pub enum Unification {
    /// A single query to be unified with the database.
    Query(Query),
    /// An entire sub-rule of unifications to be made.
    Body(Body),
}

impl Unification {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::unification);
        let pair = just!(pair.into_inner());
        let unification = match pair.as_rule() {
            Rule::assumption => Self::from_assumption(pair, context)?,
            Rule::unification => Self::Query(Query::from_predicate(pair, context)?),
            Rule::procession => Self::Body(Body::new_inner(pair, context)?),
            _ => unreachable!(),
        };
        Some(unification)
    }

    pub(crate) fn from_assumption(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::assumption);
        let mut pairs = pair.into_inner();
        let output = Pattern::new(pairs.next().unwrap(), context);
        Some(Self::Body(Body::new_evaluation(computation(
            pairs.next().unwrap(),
            context,
            output,
        )?)))
    }
}
