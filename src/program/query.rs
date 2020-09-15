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
    pub(crate) fn from_head<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Self {
        assert_eq!(pair.as_rule(), Rule::head);
        Self::new_unscoped(pair, context)
    }

    pub(crate) fn from_function_head<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Self {
        assert_eq!(pair.as_rule(), Rule::function_head);
        let mut query = Self::new_unscoped(pair, context);
        query.handle.extend_arity(Arity::Len(1.into()));
        query
    }

    fn new_unscoped<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Self {
        let mut pairs = pair.into_inner();
        let atom = context.atomizer.atomize(pairs.next().unwrap());
        let (arity, patterns) = fields(pairs.next().unwrap(), context);
        let scope = context.current_scope.join(atom);
        let handle = Handle::from_parts(scope, arity);
        Query { handle, patterns }
    }

    pub(crate) fn from_predicate<'i>(
        pair: crate::Pair<'i>,
        context: &mut Context<'i>,
    ) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        Self::new_scoped(pair, context)
    }

    pub(crate) fn from_call<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        let mut query = Self::new_scoped(pair, context)?;
        query.handle.extend_arity(Arity::Len(1.into()));
        Some(query)
    }

    fn new_scoped<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Option<Self> {
        let mut pairs = pair.into_inner();
        let scope = Scope::new(pairs.next().unwrap(), context)?;
        let (arity, patterns) = fields(pairs.next().unwrap(), context);
        let handle = Handle::from_parts(scope, arity);
        Some(Query { handle, patterns })
    }
}

fn fields<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> (Vec<Arity>, Vec<Pattern>) {
    assert_eq!(pair.as_rule(), Rule::fields);
    pair.into_inner().map(|pair| field(pair, context)).fold(
        (vec![], vec![]),
        |(mut arity, mut patterns), (name, pattern)| {
            match name {
                Some(name) => arity.push(Arity::Name(name)),
                None => {
                    if let Some(Arity::Len(len)) = arity.last_mut() {
                        *len += 1;
                    } else {
                        arity.push(Arity::Len(1.into()));
                    }
                }
            }
            patterns.push(pattern);
            (arity, patterns)
        },
    )
}

fn field<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> (Option<Atom>, Pattern) {
    assert_eq!(pair.as_rule(), Rule::field);
    let pair = just!(pair.into_inner());
    match pair.as_rule() {
        Rule::named_field => {
            let mut pairs = pair.into_inner();
            let atom = context.atomizer.atomize(pairs.next().unwrap());
            let pattern = Pattern::new(pairs.next().unwrap(), context);
            (Some(atom), pattern)
        }
        Rule::bare_field => (None, Pattern::new(just!(pair.into_inner()), context)),
        _ => unreachable!(),
    }
}
