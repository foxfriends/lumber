use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A handle to a predicate.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Handle {
    /// The path and name of the predicate or function being described
    scope: Scope,
    /// The arity of the predicate or function being described
    arity: Vec<Arity>,
}

impl Handle {
    pub fn module(&self) -> Scope {
        self.scope.drop()
    }

    pub fn head(&self) -> Self {
        Self {
            scope: Scope::default().join(self.scope.head()),
            arity: self.arity.clone(),
        }
    }

    pub fn like(&self, other: &Self) -> bool {
        self.scope.head() == other.scope.head() && self.arity == other.arity
    }

    pub(crate) fn from_parts(scope: Scope, mut arity: Vec<Arity>) -> Self {
        if arity.is_empty() {
            arity.push(Arity::Len(0.into()));
        }
        Handle { scope, arity }
    }

    pub(crate) fn binop(scope: Scope) -> Self {
        Self::from_parts(scope, vec![Arity::Len(3.into())])
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
