use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// A disjunction of conjunctions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Disjunction {
    /// Cases between which variable bindings are not shared.
    pub(crate) cases: Vec<(Conjunction, Option<Conjunction>)>,
}

impl Disjunction {
    fn conjunctions_mut(&mut self) -> impl Iterator<Item = &mut Conjunction> {
        self.cases
            .iter_mut()
            .flat_map(|(head, tail)| std::iter::once(head).chain(tail.iter_mut()))
    }

    fn conjunctions(&self) -> impl Iterator<Item = &Conjunction> {
        self.cases
            .iter()
            .flat_map(|(head, tail)| std::iter::once(head).chain(tail.iter()))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.conjunctions_mut().flat_map(Conjunction::handles_mut)
    }
}

impl Display for Disjunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, (head, tail)) in self.cases.iter().enumerate() {
            if i != 0 {
                write!(f, "; ")?;
            }
            head.fmt(f)?;
            if let Some(tail) = tail {
                write!(f, " ->> {}", tail)?;
            }
        }
        Ok(())
    }
}

impl From<ast::Disjunction> for Disjunction {
    fn from(ast: ast::Disjunction) -> Self {
        Self {
            cases: ast
                .cases
                .into_iter()
                .map(|(head, tail)| (Conjunction::from(head), tail.map(Conjunction::from)))
                .collect(),
        }
    }
}

impl Variables for Disjunction {
    fn variables(&self, vars: &mut Vec<Variable>) {
        for conjunction in self.conjunctions() {
            conjunction.variables(vars);
        }
    }
}
