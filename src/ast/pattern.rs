use super::*;
use crate::parser::Rule;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) enum Pattern {
    /// A structured pattern (unifies structurally with another query of the same name).
    Struct(Struct),
    /// A single variable (unifies with anything but only once).
    Variable(Identifier),
    /// A literal value (unifies only with itself).
    Literal(Literal),
    /// A list of patterns (unifies with a list of the same length where the patterns each
    /// unify in order).
    List(Vec<Pattern>, Option<Box<Pattern>>),
    /// A set of patterns (unifies with a set containing the same elements, ignoring order
    /// and duplicates).
    Set(Vec<Pattern>, Option<Box<Pattern>>),
    /// A wildcard (unifies with anything).
    Wildcard,
}

impl Default for Pattern {
    fn default() -> Self {
        Self::Wildcard
    }
}

impl Pattern {
    pub fn new(pair: crate::Pair, context: &mut Context) -> Self {
        assert_eq!(pair.as_rule(), Rule::pattern);
        let pair = just!(pair.into_inner());
        Self::new_inner(pair, context)
    }

    pub fn new_inner(pair: crate::Pair, context: &mut Context) -> Self {
        match pair.as_rule() {
            Rule::struct_ => Self::Struct(Struct::new(pair, context)),
            Rule::literal => Self::Literal(Literal::new(pair)),
            Rule::variable => Self::Variable(context.get_variable(pair.as_str())),
            Rule::list => {
                let mut pairs = pair.into_inner();
                let head = match pairs.next() {
                    Some(head) => head
                        .into_inner()
                        .map(|pair| Self::new(pair, context))
                        .collect(),
                    None => return Self::List(vec![], None),
                };
                let tail = pairs
                    .next()
                    .map(|pair| Box::new(Pattern::new_inner(pair, context)));
                Self::List(head, tail)
            }
            Rule::set => {
                let mut pairs = pair.into_inner();
                let head = match pairs.next() {
                    Some(head) => head
                        .into_inner()
                        .map(|pair| Self::new(pair, context))
                        .collect(),
                    None => return Self::Set(vec![], None),
                };
                let tail = pairs
                    .next()
                    .map(|pair| Box::new(Pattern::new_inner(pair, context)));
                Self::Set(head, tail)
            }
            Rule::wildcard => Self::Wildcard,
            _ => unreachable!(),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Struct(s) => Box::new(s.identifiers()),
            Self::Variable(identifier) => Box::new(std::iter::once(*identifier)),
            Self::List(head, tail) | Self::Set(head, tail) => Box::new(
                head.iter()
                    .flat_map(|pattern| pattern.identifiers())
                    .chain(tail.iter().flat_map(|pattern| pattern.identifiers())),
            ),
            _ => Box::new(std::iter::empty()),
        }
    }
}
