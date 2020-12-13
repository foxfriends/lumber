use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A disjunction of conjunctions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Disjunction {
    /// Cases between which variable bindings are not shared.
    pub(crate) cases: Vec<(Conjunction, Option<Conjunction>)>,
}

impl Disjunction {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::disjunction);
        let cases = pair
            .into_inner()
            .map(|pair| {
                let mut parts = pair
                    .into_inner()
                    .map(|pair| Conjunction::new(pair, context));
                let first = parts.next().unwrap()?;
                let second = parts.next().unwrap_or(None);
                Some((first, second))
            })
            .collect::<Option<_>>()?;
        Some(Self { cases })
    }

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

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Handle>>(&mut self, mut resolve: F) {
        self.conjunctions_mut()
            .for_each(move |conjunction| conjunction.resolve_operators(&mut resolve))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.conjunctions_mut().flat_map(Conjunction::handles_mut)
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.conjunctions().flat_map(Conjunction::identifiers)
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
