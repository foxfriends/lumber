use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A path to a defined rule.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
pub struct Scope {
    /// The library the rule is defined in, if not defined by the user.
    lib: Option<Atom>,
    /// The path to this rule, relative to the library root.
    path: Vec<Atom>,
}

impl Scope {
    pub(crate) fn new<'i>(pair: crate::Pair<'i>, context: &mut Context<'i>) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::scope);
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let mut scope = match pairs.peek().unwrap().as_rule() {
            Rule::lib => Scope {
                lib: Some(context.atomizer.atomize(pairs.next().unwrap())),
                ..Scope::default()
            },
            Rule::root => Scope::default(),
            _ => {
                let mut scope = context.current_scope.clone();
                while let Rule::up = pairs.peek().unwrap().as_rule() {
                    if scope.path.is_empty() {
                        context.error_negative_scope(span);
                        return None;
                    }
                    scope.pop();
                }
                scope
            }
        };
        for pair in pairs {
            let atom = context.atomizer.atomize(pair);
            scope.push(atom);
        }
        Some(scope)
    }

    pub(crate) fn join(&self, atom: Atom) -> Self {
        let mut path = self.path.clone();
        path.push(atom);
        Self {
            lib: self.lib.clone(),
            path,
        }
    }

    pub(crate) fn push(&mut self, atom: Atom) {
        self.path.push(atom);
    }

    pub(crate) fn pop(&mut self) {
        assert!(!self.path.is_empty(), "Attempted to pop an empty scope");
        self.path.pop();
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(lib) = &self.lib {
            write!(f, "@{}::", lib)?;
        }
        self.path
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("::")
            .fmt(f)
    }
}
