use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A handle to a predicate.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Handle {
    /// The path and name of the predicate or function being described
    scope: Scope,
    /// The arity of the predicate or function being described
    arity: Vec<Arity>,
}

impl Handle {
    pub(crate) fn from_parts(scope: Scope, mut arity: Vec<Arity>) -> Self {
        if arity.is_empty() {
            arity.push(Arity::Len(0.into()));
        }
        Handle { scope, arity }
    }

    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Self {
        Self::new_in_scope(context.current_scope.clone(), pair, context)
    }

    pub(crate) fn new_in_scope(mut scope: Scope, pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::handle);
        let mut pairs = pair.into_inner();
        let atom = context.atomizer.atomize(pairs.next().unwrap());
        scope.push(atom);
        let arity = pairs.map(|pair| Arity::new(pair, context)).collect();
        Self { scope, arity }
    }

    pub(crate) fn extend_arity(&mut self, arity: Arity) {
        match arity {
            Arity::Name(..) => self.arity.push(arity),
            Arity::Len(len) => {
                if let Some(Arity::Len(prev)) = self.arity.last_mut() {
                    *prev += len;
                } else {
                    self.arity.push(Arity::Len(len));
                }
            }
        }
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.scope.fmt(f)?;
        for arity in &self.arity {
            arity.fmt(f)?;
        }
        Ok(())
    }
}
