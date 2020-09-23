use super::*;
use crate::parser::Rule;
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Display, Formatter};

/// A path to a defined rule.
#[derive(Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct Scope {
    /// The library the rule is defined in, if not defined by the user.
    lib: Option<Atom>,
    /// The path to this rule, relative to the library root.
    path: Vec<Atom>,
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.lib != other.lib {
            return None;
        }
        if self.path.len() < other.path.len() {
            let all_equal = self.path.iter().zip(other.path.iter()).all(|(a, b)| a == b);
            if all_equal {
                Some(Ordering::Greater)
            } else {
                None
            }
        } else if self.path.len() > other.path.len() {
            let all_equal = other.path.iter().zip(self.path.iter()).all(|(a, b)| a == b);
            if all_equal {
                Some(Ordering::Less)
            } else {
                None
            }
        } else if self.path == other.path {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl Scope {
    pub(crate) fn builtin(name: &'static str, context: &mut Context) -> Self {
        Self {
            lib: Some(context.atomizer.atomize_str("core")),
            path: vec![context.atomizer.atomize_str(name)],
        }
    }

    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::scope);
        let mut pairs = pair.into_inner();
        let mut scope = match pairs.peek().unwrap().as_rule() {
            Rule::scope_prefix => Scope::new_prefix(pairs.next().unwrap(), context)?,
            Rule::atom => context.current_scope.clone(),
            _ => unreachable!(),
        };
        for pair in pairs {
            let atom = context.atomizer.atomize(pair);
            scope.push(atom);
        }
        Some(scope)
    }

    fn new_prefix(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        match pairs.peek().unwrap().as_rule() {
            Rule::lib => Some(Scope {
                lib: Some(context.atomizer.atomize(pairs.next().unwrap())),
                ..Scope::default()
            }),
            Rule::root => Some(Scope::default()),
            Rule::up => {
                let mut scope = context.current_scope.clone();
                while pairs.next().is_some() {
                    if scope.path.is_empty() {
                        context.error_negative_scope(span);
                        return None;
                    }
                    scope.pop();
                }
                Some(scope)
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn new_module_path(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::module_path);
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::scope => Self::new(pair, context),
            Rule::scope_prefix => Self::new_prefix(pair, context),
            _ => unreachable!(),
        }
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

    pub(crate) fn drop(&self) -> Self {
        let mut scope = self.clone();
        scope.pop();
        scope
    }

    pub(crate) fn head(&self) -> Atom {
        assert!(
            !self.path.is_empty(),
            "Attempted to get the head of an empty scope"
        );
        self.path.last().unwrap().clone()
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(lib) = &self.lib {
            write!(f, "@{}::", lib)?;
        }
        if self.path.is_empty() {
            "~".fmt(f)
        } else {
            self.path
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("::")
                .fmt(f)
        }
    }
}

impl<'a> IntoIterator for &'a Scope {
    type Item = &'a Atom;
    type IntoIter = <&'a [Atom] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.path).into_iter()
    }
}
