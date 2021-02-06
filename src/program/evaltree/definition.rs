use super::*;
use crate::ast;
use std::iter::FromIterator;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Default, Clone, Debug)]
pub(crate) struct Definition(Vec<(Head, RuleKind, Option<Body>)>);

impl Definition {
    pub fn iter(&self) -> impl Iterator<Item = &(Head, RuleKind, Option<Body>)> {
        self.0.iter()
    }

    pub fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.0.iter_mut().filter_map(|(_, _, body)| body.as_mut())
    }

    pub fn merge(&mut self, mut other: Definition) {
        self.0.append(&mut other.0);
    }
}

impl From<ast::Definition> for Definition {
    fn from(ast: ast::Definition) -> Self {
        Self(
            ast.into_iter()
                .map(|(head, kind, body)| (head.into(), kind, body.map(Into::into)))
                .collect(),
        )
    }
}

impl FromIterator<Self> for Definition {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        iter.into_iter().fold(Self::default(), |mut acc, def| {
            acc.merge(def);
            acc
        })
    }
}
