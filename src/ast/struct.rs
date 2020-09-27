use super::*;
use crate::parser::Rule;

/// A structured value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    name: Atom,
    /// The shape of the struct
    arity: Vec<Arity>,
    /// The values in the struct
    fields: Vec<Pattern>,
}

impl Struct {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_);
        let mut pairs = pair.into_inner();
        let name = context.atomizer.atomize(pairs.next().unwrap());
        let (arity, patterns) = pairs
            .next()
            .map(|pair| fields(pair, context))
            .unwrap_or((vec![Arity::Len(0.into())], vec![]));
        Self {
            name,
            arity,
            fields: patterns,
        }
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.fields.iter().flat_map(|pattern| pattern.identifiers())
    }
}
