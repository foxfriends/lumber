use super::*;
use crate::parser::Rule;

/// A structured value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    pub(crate) name: Atom,
    /// The tuple portion of the struct
    pub(crate) patterns: Vec<Pattern>,
    /// The record portion of the struct
    pub(crate) fields: Params,
}

impl Struct {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_);
        let mut pairs = pair.into_inner();
        let name = Atom::new(pairs.next().unwrap());
        let (patterns, fields) = pairs
            .next()
            .map(|pair| Params::new(pair, context))
            .unwrap_or((vec![], Params::default()));
        Self::from_parts(name, patterns, fields)
    }

    pub fn from_parts(name: Atom, patterns: Vec<Pattern>, fields: Params) -> Self {
        Self {
            name,
            patterns,
            fields,
        }
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.patterns
            .iter()
            .flat_map(|pattern| pattern.identifiers())
    }
}
