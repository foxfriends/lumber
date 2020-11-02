use super::*;
use crate::parser::Rule;

/// A structured value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    pub(crate) name: Atom,
    /// The shape of the struct
    pub(crate) arity: Arity,
    /// The values in the struct
    pub(crate) fields: Vec<Pattern>,
}

impl Struct {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_);
        let mut pairs = pair.into_inner();
        let name = Atom::new(pairs.next().unwrap());
        let (arity, patterns) = pairs
            .next()
            .map(|pair| fields(pair, context))
            .unwrap_or((Arity::default(), vec![]));
        Self::from_parts(name, arity, patterns)
    }

    pub fn from_parts(name: Atom, mut arity: Arity, mut fields: Vec<Pattern>) -> Self {
        arity.sort(&mut fields);
        Self {
            name,
            arity,
            fields,
        }
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.fields.iter().flat_map(|pattern| pattern.identifiers())
    }
}
