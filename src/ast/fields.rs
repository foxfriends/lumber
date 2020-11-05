use super::*;
use crate::parser::Rule;
use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Clone, Hash, Eq, PartialEq, Default, Debug)]
pub(crate) struct Fields {
    fields: BTreeMap<Atom, Vec<Pattern>>,
}

impl Fields {
    pub fn new(pair: crate::Pair, context: &mut Context) -> (Vec<Pattern>, Self) {
        assert_eq!(pair.as_rule(), Rule::fields);
        let mut pairs = pair.into_inner().peekable();
        let tuple = if pairs.peek().unwrap().as_rule() == Rule::bare_fields {
            pairs
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| Pattern::new(pair, context))
                .collect()
        } else {
            vec![]
        };
        let fields = pairs
            .next()
            .map(|pair| Self::from_named(pair, context))
            .unwrap_or_else(Self::default);
        (tuple, fields)
    }

    pub fn from_named(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::named_fields);
        let mut fields = BTreeMap::default();
        for pair in pair.into_inner() {
            let mut pairs = pair.into_inner();
            let name = Atom::new(pairs.next().unwrap());
            let values = just!(Rule::bare_fields, pairs)
                .into_inner()
                .map(|pair| Pattern::new(pair, context))
                .collect::<Vec<_>>();
            fields.insert(name, values);
        }
        Self { fields }
    }

    pub fn append(&mut self, other: &mut Self) {
        self.fields.append(&mut other.fields);
    }

    pub fn similar(&self, other: &Self) -> bool {
        self.fields.len() == other.fields.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(lhs, rhs)| lhs.0 == rhs.0 && lhs.1.len() == rhs.1.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Atom, &Vec<Pattern>)> {
        self.fields.iter()
    }
}

impl Into<BTreeMap<Atom, Vec<Pattern>>> for Fields {
    fn into(self) -> BTreeMap<Atom, Vec<Pattern>> {
        self.fields
    }
}

impl From<BTreeMap<Atom, Vec<Pattern>>> for Fields {
    fn from(fields: BTreeMap<Atom, Vec<Pattern>>) -> Self {
        Self { fields }
    }
}

impl<T> FromIterator<T> for Fields
where
    BTreeMap<Atom, Vec<Pattern>>: FromIterator<T>,
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
    type Item = <BTreeMap<Atom, Vec<Pattern>> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Atom, Vec<Pattern>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}
