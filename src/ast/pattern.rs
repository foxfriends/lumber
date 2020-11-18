use super::*;
use crate::parser::Rule;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Debug)]
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
    #[cfg(feature = "builtin-sets")]
    Set(Vec<Pattern>, Option<Box<Pattern>>),
    /// A record, containing a set of fields.
    Record(Fields, Option<Box<Pattern>>),
    /// A wildcard (unifies with anything).
    Wildcard,
    /// An unknown Rust value.
    Any(Rc<Box<dyn Any>>),
}

impl Default for Pattern {
    fn default() -> Self {
        Self::Wildcard
    }
}

impl Eq for Pattern {}
impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Pattern::Struct(lhs), Pattern::Struct(rhs)) => lhs == rhs,
            (Pattern::Variable(lhs), Pattern::Variable(rhs)) => lhs == rhs,
            (Pattern::Literal(lhs), Pattern::Literal(rhs)) => lhs == rhs,
            #[cfg(feature = "builtin-sets")]
            (Pattern::Set(lhs, ltail), Pattern::Set(rhs, rtail)) => lhs == rhs && ltail == rtail,
            (Pattern::List(lhs, ltail), Pattern::List(rhs, rtail)) => lhs == rhs && ltail == rtail,
            (Pattern::Record(lhs, ltail), Pattern::Record(rhs, rtail)) => {
                lhs == rhs && ltail == rtail
            }
            (Pattern::Any(lhs), Pattern::Any(rhs)) => Rc::ptr_eq(lhs, rhs),
            (Pattern::Wildcard, Pattern::Wildcard) => true,
            _ => false,
        }
    }
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Pattern::Struct(value) => value.hash(hasher),
            Pattern::Variable(value) => value.hash(hasher),
            Pattern::Literal(value) => value.hash(hasher),
            #[cfg(feature = "builtin-sets")]
            Pattern::Set(value, tail) => (value, tail).hash(hasher),
            Pattern::List(value, tail) => (value, tail).hash(hasher),
            Pattern::Record(value, tail) => (value, tail).hash(hasher),
            Pattern::Any(value) => Rc::as_ptr(value).hash(hasher),
            Pattern::Wildcard => ().hash(hasher),
        }
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
                let tail = pairs.next().map(|pair| match pair.into_inner().next() {
                    Some(pair) => Box::new(Pattern::new_inner(pair, context)),
                    None => Box::new(Pattern::Wildcard),
                });
                Self::List(head, tail)
            }
            #[cfg(not(feature = "builtin-sets"))]
            Rule::set => {
                unimplemented!("builtin-sets is not enabled, so set pattern syntax cannot be used.")
            }
            #[cfg(feature = "builtin-sets")]
            Rule::set => {
                let mut pairs = pair.into_inner();
                let head = match pairs.next() {
                    Some(head) => head
                        .into_inner()
                        .map(|pair| Self::new(pair, context))
                        .collect(),
                    None => return Self::Set(vec![], None),
                };
                let tail = pairs.next().map(|pair| match pair.into_inner().next() {
                    Some(pair) => Box::new(Pattern::new_inner(pair, context)),
                    None => Box::new(Pattern::Wildcard),
                });
                Self::Set(head, tail)
            }
            Rule::record => {
                let mut pairs = pair.into_inner();
                let head = match pairs.next() {
                    Some(head) => Fields::new(head, context),
                    None => return Self::Record(Fields::default(), None),
                };
                let tail = pairs.next().map(|pair| match pair.into_inner().next() {
                    Some(pair) => Box::new(Pattern::new_inner(pair, context)),
                    None => Box::new(Pattern::Wildcard),
                });
                Self::Record(head, tail)
            }
            Rule::wildcard => Self::Wildcard,
            _ => unreachable!(),
        }
    }

    /// Identifiers for every placeholder value in this pattern, including wildcards.
    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Struct(s) => Box::new(s.identifiers()),
            Self::Variable(identifier) => Box::new(std::iter::once(identifier.clone())),
            Self::List(head, tail) => Box::new(
                head.iter()
                    .flat_map(|pattern| pattern.identifiers())
                    .chain(tail.iter().flat_map(|pattern| pattern.identifiers())),
            ),
            Self::Record(head, tail) => Box::new(
                head.iter()
                    .flat_map(|(_, pattern)| pattern.identifiers())
                    .chain(tail.iter().flat_map(|pattern| pattern.identifiers())),
            ),
            #[cfg(feature = "builtin-sets")]
            Self::Set(head, tail) => Box::new(
                head.iter()
                    .flat_map(|pattern| pattern.identifiers())
                    .chain(tail.iter().flat_map(|pattern| pattern.identifiers())),
            ),
            // TODO: give these unique names, and allow user to specify wildcard names.
            Self::Wildcard => Box::new(std::iter::once(Identifier::wildcard("_".to_owned()))),
            _ => Box::new(std::iter::empty()),
        }
    }
}
