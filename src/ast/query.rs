use super::*;
use crate::parser::Rule;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) struct Query {
    /// The shape of this query.
    pub(crate) handle: Handle,
    /// The patterns in each field.
    pub(crate) patterns: Vec<Pattern>,
}

impl AsRef<Handle> for Query {
    fn as_ref(&self) -> &Handle {
        &self.handle
    }
}

impl AsMut<Handle> for Query {
    fn as_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }
}

impl Query {
    pub fn new(handle: Handle, patterns: Vec<Pattern>) -> Self {
        Self { handle, patterns }
    }
}

impl Query {
    pub fn from_head(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::head);
        Self::new_unscoped(pair, context)
    }

    pub fn from_function_head(pair: crate::Pair, context: &mut Context, output: Pattern) -> Self {
        assert_eq!(pair.as_rule(), Rule::function_head);
        let mut query = Self::new_unscoped(pair, context);
        query.handle.arity.extend_len();
        query.patterns.push(output);
        query
    }

    fn new_unscoped(pair: crate::Pair, context: &mut Context) -> Self {
        let mut pairs = pair.into_inner();
        let atom = Atom::new(pairs.next().unwrap());
        let scope = context.current_scope.join(atom);
        let (arity, patterns) = pairs
            .next()
            .map(|pair| params(pair, context))
            .unwrap_or((Arity::default(), vec![]));
        let handle = Handle::from_parts(scope, arity);
        Query { handle, patterns }
    }

    pub fn from_predicate(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        Self::new_scoped(pair, context)
    }

    pub fn from_call(pair: crate::Pair, context: &mut Context, output: Pattern) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::call);
        let mut query = Self::new_scoped(pair, context)?;
        query.handle.arity.extend_len();
        query.patterns.push(output);
        Some(query)
    }

    fn new_scoped(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        let mut pairs = pair.into_inner();
        let scope = Scope::new(pairs.next().unwrap(), context)?;
        let (arity, patterns) = pairs
            .next()
            .map(|pair| params(pair, context))
            .unwrap_or((Arity::default(), vec![]));
        let handle = Handle::from_parts(scope, arity);
        Some(Query { handle, patterns })
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.patterns
            .iter()
            .flat_map(|pattern| pattern.identifiers())
    }
}

fn params(pair: crate::Pair, context: &mut Context) -> (Arity, Vec<Pattern>) {
    assert_eq!(pair.as_rule(), Rule::params);
    let mut pairs = pair.into_inner().peekable();
    let mut arity = Arity::default();
    let mut patterns = vec![];
    if pairs.peek().unwrap().as_rule() == Rule::bare_params {
        patterns.extend(
            pairs
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| Pattern::new(pair, context)),
        );
        arity.len = patterns.len() as u32;
    }
    match pairs.next() {
        Some(pair) => {
            assert_eq!(pair.as_rule(), Rule::named_params);
            for pair in pair.into_inner() {
                let mut pairs = pair.into_inner();
                let name = Atom::new(pairs.next().unwrap());
                let values = just!(Rule::bare_params, pairs)
                    .into_inner()
                    .map(|pair| Pattern::new(pair, context))
                    .collect::<Vec<_>>();
                arity.push(name, values.len() as u32);
                patterns.extend(values);
            }
        }
        None => {}
    }
    (arity, patterns)
}
