use super::*;
use crate::parser::Rule;

#[derive(Clone, Debug)]
pub struct Query {
    /// The shape of this query.
    handle: Handle,
    /// The patterns in each field.
    patterns: Vec<Pattern>,
}

impl AsRef<Handle> for Query {
    fn as_ref(&self) -> &Handle {
        &self.handle
    }
}

impl Query {
    pub(crate) fn from_head(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::head);
        Self::new_unscoped(pair, context)
    }

    pub(crate) fn from_function_head(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::function_head);
        let mut query = Self::new_unscoped(pair, context);
        query.handle.extend_arity(Arity::Len(1.into()));
        query
    }

    fn new_unscoped(pair: crate::Pair, context: &mut Context) -> Self {
        let mut pairs = pair.into_inner();
        let atom = context.atomizer.atomize(pairs.next().unwrap());
        let (arity, patterns) = fields(pairs.next().unwrap(), context);
        let scope = context.current_scope.join(atom);
        let handle = Handle::from_parts(scope, arity);
        Query { handle, patterns }
    }

    pub(crate) fn from_predicate(
        pair: crate::Pair,
        context: &mut Context,
    ) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        Self::new_scoped(pair, context)
    }

    pub(crate) fn from_call(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        let mut query = Self::new_scoped(pair, context)?;
        query.handle.extend_arity(Arity::Len(1.into()));
        Some(query)
    }

    fn new_scoped(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        let mut pairs = pair.into_inner();
        let scope = Scope::new(pairs.next().unwrap(), context)?;
        let (arity, patterns) = fields(pairs.next().unwrap(), context);
        let handle = Handle::from_parts(scope, arity);
        Some(Query { handle, patterns })
    }
}
