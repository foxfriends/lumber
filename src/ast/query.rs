use super::*;
use crate::parser::Rule;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) struct Query {
    /// The shape of this query.
    pub(crate) handle: Handle,
    /// The patterns in each field.
    pub(crate) patterns: Vec<Pattern>,
}

impl Query {
    pub fn new(handle: Handle, patterns: Vec<Pattern>) -> Self {
        Self { handle, patterns }
    }

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

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.patterns
            .iter()
            .flat_map(|pattern| pattern.identifiers())
    }

    pub fn check_variables(&self, context: &mut Context) {
        let counts = self
            .identifiers()
            .filter(|ident| !ident.is_wildcard())
            .fold(
                HashMap::<Identifier, usize>::default(),
                |mut map, identifier| {
                    *map.entry(identifier).or_default() += 1;
                    map
                },
            );

        for (identifier, count) in counts {
            if count <= 1 {
                context.error_singleton_variable(self.as_ref(), identifier.name());
            }
        }
    }
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

impl Display for Query {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.handle.scope.fmt(f)?;
        if self.patterns.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        for (i, pattern) in self.patterns.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            if i < self.handle.arity.len as usize {
                pattern.fmt(f)?;
            } else {
                let mut j = self.handle.arity.len;
                for (name, len) in &self.handle.arity.fields {
                    if i as u32 == j {
                        write!(f, "{}: {}", name, pattern)?;
                        break;
                    }
                    j += len;
                    if (i as u32) < j {
                        pattern.fmt(f)?;
                        break;
                    }
                }
            }
        }
        write!(f, ")")
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
    if let Some(pair) = pairs.next() {
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
    (arity, patterns)
}
