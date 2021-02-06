use super::*;
use crate::parser::Rule;
use std::collections::BTreeMap;

#[derive(Clone, Hash, Eq, PartialEq, Default, Debug)]
pub(crate) struct Fields {
    pub fields: BTreeMap<Atom, Pattern>,
}

impl Fields {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::fields);
        let fields = pair
            .into_inner()
            .map(|pair| {
                let mut pairs = pair.into_inner();
                let atom = Atom::new(pairs.next().unwrap());
                let pattern = Pattern::new(pairs.next().unwrap(), context);
                (atom, pattern)
            })
            .collect();
        Self { fields }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Atom, &Pattern)> {
        self.fields.iter()
    }
}

impl IntoIterator for Fields {
    type Item = <BTreeMap<Atom, Pattern> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Atom, Pattern> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}
