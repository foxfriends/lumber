use super::*;
use crate::parser::Rule;

/// A named container, which can optionally contain a value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Struct {
    /// The tag of the struct
    pub(crate) name: Atom,
    /// The contents of the struct
    pub(crate) contents: Option<Box<Pattern>>,
}

impl Struct {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_);
        let mut pairs = pair.into_inner();
        let name = Atom::new(pairs.next().unwrap());
        let contents = pairs.next().map(|pair| match pair.as_rule() {
            Rule::pattern => Box::new(Pattern::new(pair, context)),
            _ => Box::new(Pattern::new_inner(pair, context)),
        });
        Self::from_parts(name, contents)
    }

    pub fn from_parts(name: Atom, contents: Option<Box<Pattern>>) -> Self {
        Self { name, contents }
    }

    pub fn identifiers<'a>(&'a self) -> impl Iterator<Item = Identifier> + 'a {
        self.contents
            .iter()
            .flat_map(|pattern| pattern.identifiers())
    }
}
