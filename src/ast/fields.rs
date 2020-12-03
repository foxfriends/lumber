use super::*;
use crate::parser::Rule;
use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Clone, Hash, Eq, PartialEq, Default, Debug)]
pub(crate) struct Fields {
    fields: BTreeMap<Atom, Pattern>,
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

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn append(&mut self, other: &mut Self) {
        self.fields.append(&mut other.fields);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Atom, &Pattern)> {
        self.fields.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Atom, &mut Pattern)> {
        self.fields.iter_mut()
    }
}

impl Into<BTreeMap<Atom, Pattern>> for Fields {
    fn into(self) -> BTreeMap<Atom, Pattern> {
        self.fields
    }
}

impl From<BTreeMap<Atom, Pattern>> for Fields {
    fn from(fields: BTreeMap<Atom, Pattern>) -> Self {
        Self { fields }
    }
}

impl<T> FromIterator<T> for Fields
where
    BTreeMap<Atom, Pattern>: FromIterator<T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Fields {
            fields: BTreeMap::from_iter(iter),
        }
    }
}

impl IntoIterator for Fields {
    type Item = <BTreeMap<Atom, Pattern> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Atom, Pattern> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}
