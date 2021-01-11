#![allow(clippy::redundant_allocation)]
use super::*;
use crate::parser::Rule;
use std::any::Any;
use std::fmt::{self, Display, Formatter};
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
    /// Indexing into a list.
    ListIndex(Box<Pattern>, Box<Pattern>),
    /// A set of patterns (unifies with a set containing the same elements, ignoring order
    /// and duplicates).
    #[cfg(feature = "builtin-sets")]
    Set(Vec<Pattern>, Option<Box<Pattern>>),
    /// A record, containing a set of fields.
    Record(Fields, Option<Box<Pattern>>),
    /// Indexing into a record.
    RecordIndex(Box<Pattern>, Box<Pattern>),
    /// A wildcard (unifies with anything).
    Wildcard(Identifier),
    /// An unknown Rust value.
    Any(Rc<Box<dyn Any>>),
    /// A value that must already be bound, at the time of checking (not wildcard).
    Bound,
    /// A value that must already not be bound, at the time of checking (wildcard only).
    Unbound,
    /// A value that must match multiple patterns.
    All(Vec<Pattern>),
}

impl Default for Pattern {
    fn default() -> Self {
        Self::Wildcard(Identifier::wildcard("_default"))
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
            (Pattern::ListIndex(llist, lind), Pattern::ListIndex(rlist, rind)) => {
                llist == rlist && lind == rind
            }
            (Pattern::Record(lhs, ltail), Pattern::Record(rhs, rtail)) => {
                lhs == rhs && ltail == rtail
            }
            (Pattern::RecordIndex(lrec, lind), Pattern::RecordIndex(rrec, rind)) => {
                lrec == rrec && lind == rind
            }
            (Pattern::Any(lhs), Pattern::Any(rhs)) => Rc::ptr_eq(lhs, rhs),
            (Pattern::Wildcard(lhs), Pattern::Wildcard(rhs)) => lhs == rhs,
            (Pattern::Bound, Pattern::Bound) => true,
            (Pattern::Unbound, Pattern::Unbound) => true,
            (Pattern::All(lhs), Pattern::All(rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Pattern::Struct(value) => ("struct", value).hash(hasher),
            Pattern::Variable(value) => ("variable", value).hash(hasher),
            Pattern::Literal(value) => ("literal", value).hash(hasher),
            #[cfg(feature = "builtin-sets")]
            Pattern::Set(value, tail) => ("set", value, tail).hash(hasher),
            Pattern::List(value, tail) => ("list", value, tail).hash(hasher),
            Pattern::ListIndex(list, index) => ("listindex", list, index).hash(hasher),
            Pattern::Record(value, tail) => ("record", value, tail).hash(hasher),
            Pattern::RecordIndex(record, index) => ("recordindex", record, index).hash(hasher),
            Pattern::Any(value) => ("any", Rc::as_ptr(value)).hash(hasher),
            Pattern::Wildcard(value) => ("wildcard", value).hash(hasher),
            Pattern::Bound => "bound".hash(hasher),
            Pattern::Unbound => "unbound".hash(hasher),
            Pattern::All(patterns) => ("all", patterns).hash(hasher),
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
            Rule::value_pattern => Self::new_inner(just!(pair.into_inner()), context),
            Rule::value => Self::new_inner(just!(pair.into_inner()), context),
            Rule::bound_pattern => match pair.into_inner().next() {
                Some(pair) => {
                    let inner = Self::new_inner(pair, context);
                    Self::All(vec![Self::Bound, inner])
                }
                None => Self::Bound,
            },
            Rule::unbound_pattern => match pair.into_inner().next() {
                Some(pair) => {
                    let inner = Self::new_inner(pair, context);
                    Self::All(vec![Self::Unbound, inner])
                }
                None => Self::Unbound,
            },
            Rule::list_index => {
                let mut pairs = pair.into_inner();
                let list = Self::new_inner(pairs.next().unwrap(), context);
                let index = Self::new_inner(pairs.next().unwrap(), context);
                Self::ListIndex(Box::new(list), Box::new(index))
            }
            Rule::record_index => {
                let mut pairs = pair.into_inner();
                let record = Self::new_inner(pairs.next().unwrap(), context);
                let index = Self::new_inner(pairs.next().unwrap(), context);
                Self::RecordIndex(Box::new(record), Box::new(index))
            }
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
                    None => Box::new(Pattern::Wildcard(Identifier::wildcard("_list_tail"))),
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
                    None => Box::new(Pattern::Wildcard(Identifier::wildcard("_set_tail"))),
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
                    None => Box::new(Pattern::Wildcard(Identifier::wildcard("_record_tail"))),
                });
                Self::Record(head, tail)
            }
            Rule::wildcard => Self::Wildcard(Identifier::wildcard(pair.as_str())),
            rule => unreachable!("unexpected {:?}", rule),
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
            Self::RecordIndex(col, ind) | Self::ListIndex(col, ind) => {
                Box::new(col.identifiers().chain(ind.identifiers()))
            }
            #[cfg(feature = "builtin-sets")]
            Self::Set(head, tail) => Box::new(
                head.iter()
                    .flat_map(|pattern| pattern.identifiers())
                    .chain(tail.iter().flat_map(|pattern| pattern.identifiers())),
            ),
            Self::Wildcard(identifier) => Box::new(std::iter::once(identifier.clone())),
            Self::All(patterns) => {
                Box::new(patterns.iter().flat_map(|pattern| pattern.identifiers()))
            }
            _ => Box::new(std::iter::empty()),
        }
    }

    /// Identifiers for every placeholder value in this pattern, including wildcards.
    pub fn identifiers_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Identifier> + 'a> {
        match self {
            Self::Struct(s) => Box::new(s.identifiers_mut()),
            Self::Variable(identifier) => Box::new(std::iter::once(identifier)),
            Self::List(head, tail) => Box::new(
                head.iter_mut()
                    .flat_map(|pattern| pattern.identifiers_mut())
                    .chain(
                        tail.iter_mut()
                            .flat_map(|pattern| pattern.identifiers_mut()),
                    ),
            ),
            Self::Record(head, tail) => Box::new(
                head.iter_mut()
                    .flat_map(|(_, pattern)| pattern.identifiers_mut())
                    .chain(
                        tail.iter_mut()
                            .flat_map(|pattern| pattern.identifiers_mut()),
                    ),
            ),
            Self::RecordIndex(col, ind) | Self::ListIndex(col, ind) => {
                Box::new(col.identifiers_mut().chain(ind.identifiers_mut()))
            }
            #[cfg(feature = "builtin-sets")]
            Self::Set(head, tail) => Box::new(
                head.iter_mut()
                    .flat_map(|pattern| pattern.identifiers_mut())
                    .chain(
                        tail.iter_mut()
                            .flat_map(|pattern| pattern.identifiers_mut()),
                    ),
            ),
            Self::Wildcard(identifier) => Box::new(std::iter::once(identifier)),
            Self::All(patterns) => Box::new(
                patterns
                    .iter_mut()
                    .flat_map(|pattern| pattern.identifiers_mut()),
            ),
            _ => Box::new(std::iter::empty()),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::List(..) | Self::Record(..))
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Self::Wildcard(..))
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Pattern::Literal(lit) => lit.fmt(f),
            #[cfg(feature = "builtin-sets")]
            Pattern::Set(head, tail) => todo!(),
            Pattern::List(head, tail) => {
                write!(f, "[")?;
                for (i, pattern) in head.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    pattern.fmt(f)?;
                }
                match tail {
                    Some(tail) if tail.is_wildcard() => write!(f, ", ..]"),
                    Some(tail) => write!(f, ", ..{}]", tail),
                    None => write!(f, "]"),
                }
            }
            Pattern::Record(head, tail) => {
                write!(f, "{{ ")?;
                for (i, (key, pattern)) in head.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, pattern)?;
                }
                match tail {
                    Some(tail) if tail.is_wildcard() => write!(f, ", .. }}"),
                    Some(tail) => write!(f, ", ..{} }}", tail),
                    None => write!(f, " }}"),
                }
            }
            Self::RecordIndex(record, ind) => {
                write!(f, "{} {{ {} }}", record, ind)
            }
            Self::ListIndex(list, ind) => {
                write!(f, "{} [ {} ]", list, ind)
            }
            Pattern::Struct(structure) => structure.fmt(f),
            Pattern::Any(any) => write!(f, "[{:?}]", Rc::as_ptr(any)),
            Pattern::Variable(var) => var.fmt(f),
            Pattern::Wildcard(..) => "_".fmt(f),
            Pattern::Bound => "!".fmt(f),
            Pattern::Unbound => "?".fmt(f),
            Pattern::All(inner) => {
                for pat in inner {
                    pat.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}
