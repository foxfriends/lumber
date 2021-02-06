use super::*;

/// The definition of a rule. A predicate may be defined multiple times with disjoint
/// heads and distinct bodies.
#[derive(Default, Clone, Debug)]
pub(crate) struct Definition(Vec<(Head, RuleKind, Option<Body>)>);

impl Definition {
    pub fn insert(&mut self, head: Head, kind: RuleKind, body: Option<Body>) {
        self.0.push((head, kind, body));
    }

    pub fn bodies_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.0.iter_mut().filter_map(|(_, _, body)| body.as_mut())
    }

    pub fn resolve_handles<F: FnMut(&Handle) -> Option<Handle>>(&mut self, mut resolve: F) {
        self.bodies_mut()
            .for_each(move |body| body.resolve_handles(&mut resolve));
    }

    pub fn resolve_operators<F: FnMut(&OpKey) -> Option<Operator>>(&mut self, mut resolve: F) {
        self.bodies_mut()
            .for_each(move |body| body.resolve_operators(&mut resolve));
    }
}

impl IntoIterator for Definition {
    type Item = <Vec<(Head, RuleKind, Option<Body>)> as IntoIterator>::Item;
    type IntoIter = <Vec<(Head, RuleKind, Option<Body>)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
