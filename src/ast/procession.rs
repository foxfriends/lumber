use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

/// A sequence of narrowing steps.
#[derive(Default, Clone, Debug)]
pub(crate) struct Procession {
    /// Steps after which backtracking is skipped.
    pub(crate) steps: Vec<Step>,
}

impl Procession {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::procession);
        let steps = pair
            .into_inner()
            .map(|pair| Step::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { steps })
    }

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Handle>>(&mut self, mut resolve: F) {
        self.steps
            .iter_mut()
            .for_each(move |step| step.resolve_operators(&mut resolve))
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.steps.iter_mut().flat_map(|step| step.handles_mut())
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.steps.iter().flat_map(|step| step.identifiers())
    }
}

impl Display for Procession {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, step) in self.steps.iter().enumerate() {
            if i != 0 {
                write!(f, " -> ")?;
            }
            step.fmt(f)?;
        }
        Ok(())
    }
}
