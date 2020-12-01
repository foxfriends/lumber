use super::*;
use crate::parser::Rule;

/// A disjunction of conjunctions.
#[derive(Default, Clone, Debug)]
pub(crate) struct Branch {
    /// Steps between which backtracking is prevented.
    pub(crate) steps: Vec<Conjunction>,
}

impl Branch {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Option<Self> {
        assert_eq!(pair.as_rule(), Rule::branch);
        let steps = pair
            .into_inner()
            .map(|pair| Conjunction::new(pair, context))
            .collect::<Option<_>>()?;
        Some(Self { steps })
    }

    pub fn handles_mut(&mut self) -> impl Iterator<Item = &mut Handle> {
        self.steps.iter_mut().flat_map(|step| step.handles_mut())
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> + '_ {
        self.steps.iter().flat_map(|step| step.identifiers())
    }
}
