use super::*;
use crate::parser::Rule;

/// A structured value.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Struct {
    /// The tag of the struct
    name: Atom,
    /// The shape of the struct
    arity: Vec<Arity>,
    /// The values in the struct
    fields: Vec<Pattern>,
}

impl Struct {
    pub(crate) fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_);
        let mut pairs = pair.into_inner();
        let name = context.atomizer.atomize(pairs.next().unwrap());
        let (arity, patterns) = fields(pairs.next().unwrap(), context);
        Self {
            name,
            arity,
            fields: patterns,
        }
    }
}
