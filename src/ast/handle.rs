use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A handle to a predicate.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Handle {
    /// The path and name of the predicate or function being described
    pub(crate) scope: Scope,
    /// The arity of the predicate or function being described
    pub(crate) arity: Arity,
}

pub trait AsHandle {
    #[doc(hidden)]
    fn as_handle(&self) -> crate::Result<Handle>;
}

impl AsHandle for &str {
    fn as_handle(&self) -> crate::Result<Handle> {
        let pair = just!(
            Rule::external_handle,
            crate::parser::Parser::parse_handle(self)?
        );
        let mut pairs = pair.into_inner();
        let mut scope = Scope::default();
        while Rule::atom == pairs.peek().unwrap().as_rule() {
            scope.push(Atom::new(pairs.next().unwrap()));
        }
        let arity = Arity::new(pairs.next().unwrap());
        assert_eq!(Rule::EOI, pairs.next().unwrap().as_rule());
        Ok(Handle { scope, arity })
    }
}

impl Handle {
    pub(crate) fn library(&self) -> &[Atom] {
        self.scope.library()
    }

    pub(crate) fn module(&self) -> Scope {
        self.scope.drop()
    }

    pub(crate) fn add_lib(&mut self, lib: Atom) {
        self.scope.add_lib(lib);
    }

    pub(crate) fn head(&self) -> Self {
        Self {
            scope: Scope::default().join(self.scope.head()),
            arity: self.arity.clone(),
        }
    }

    pub(crate) fn relocate(&self, scope: &Scope) -> Self {
        Self {
            scope: scope.join(self.scope.head()),
            arity: self.arity.clone(),
        }
    }

    pub(crate) fn like(&self, other: &Self) -> bool {
        self.scope.head() == other.scope.head() && self.arity == other.arity
    }

    pub(crate) fn can_alias(&self, other: &Self) -> bool {
        self.arity.can_alias(&other.arity)
    }

    pub(crate) fn from_parts(scope: Scope, arity: Arity) -> Self {
        Handle { scope, arity }
    }

    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Self {
        Self::new_in_scope(context.current_scope.clone(), pair)
    }

    pub(crate) fn new_in_scope(mut scope: Scope, pair: crate::Pair) -> Self {
        assert_eq!(pair.as_rule(), Rule::handle);
        let mut pairs = pair.into_inner();
        let atom = Atom::new(pairs.next().unwrap());
        scope.push(atom);
        let arity = Arity::new(pairs.next().unwrap());
        Self { scope, arity }
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.scope.fmt(f)?;
        self.arity.fmt(f)
    }
}
