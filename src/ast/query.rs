use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) struct Query {
    /// The shape of this query.
    pub(crate) handle: Handle,
    /// The args in each field.
    pub(crate) args: Vec<Expression>,
}

impl Query {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::predicate);
        let mut pairs = pair.into_inner();
        let scope = Scope::new(pairs.next().unwrap(), context)?;
        let (arity, args) = pairs
            .next()
            .map(|pair| arguments(pair, context))
            .unwrap_or(Some((Arity::default(), vec![])))?;
        let handle = Handle::from_parts(scope, arity);
        Some(Query { handle, args })
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.args.iter().flat_map(|pattern| pattern.identifiers())
    }

    pub fn args_mut(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.args.iter_mut()
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn args(&self) -> &[Expression] {
        &self.args
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
        if self.args.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        for (i, pattern) in self.args.iter().enumerate() {
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

impl From<Head> for Query {
    fn from(head: Head) -> Self {
        Self {
            handle: head.handle,
            args: head.patterns.into_iter().map(Expression::from).collect(),
        }
    }
}

fn arguments(pair: crate::Pair, context: &mut Context) -> Option<(Arity, Vec<Expression>)> {
    assert_eq!(pair.as_rule(), Rule::arguments);
    let mut pairs = pair.into_inner().peekable();
    let mut arity = Arity::default();
    let mut args = vec![];
    if pairs.peek().unwrap().as_rule() == Rule::bare_arguments {
        args.extend(
            pairs
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| Expression::new(pair, context))
                .collect::<Option<Vec<_>>>()?,
        );
        arity.len = args.len() as u32;
    }
    if let Some(pair) = pairs.next() {
        assert_eq!(pair.as_rule(), Rule::named_arguments);
        for pair in pair.into_inner() {
            let mut pairs = pair.into_inner();
            let name = Atom::new(pairs.next().unwrap());
            let values = just!(Rule::bare_arguments, pairs)
                .into_inner()
                .map(|pair| Expression::new(pair, context))
                .collect::<Option<Vec<_>>>()?;
            arity.push(name, values.len() as u32);
            args.extend(values);
        }
    }
    Some((arity, args))
}
