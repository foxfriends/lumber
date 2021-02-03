#![allow(clippy::redundant_allocation)]
use super::*;
use crate::ast;
use std::any::Any;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Debug)]
pub(crate) enum PatternKind {
    /// A structured pattern (unifies structurally with another query of the same name).
    Struct(Struct),
    /// A single variable (unifies with anything but only once).
    Variable(Variable),
    /// A literal value (unifies only with itself).
    Literal(Literal),
    /// A list of patterns (unifies with a list of the same length where the patterns each
    /// unify in order).
    List(Vec<Pattern>, Option<Pattern>),
    /// A record, containing a set of fields.
    Record(Fields, Option<Pattern>),
    /// An unknown Rust value.
    Any(Rc<Box<dyn Any>>),
    /// A value that must already be bound, at the time of checking (not wildcard)
    Bound,
    /// A value that must already not be bound, at the time of checking (wildcard only)
    Unbound,
    /// A value that must match multiple patterns
    All(Vec<Pattern>),
}

impl Eq for PatternKind {}
impl PartialEq for PatternKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PatternKind::Struct(lhs), PatternKind::Struct(rhs)) => lhs == rhs,
            (PatternKind::Variable(lhs), PatternKind::Variable(rhs)) => lhs == rhs,
            (PatternKind::Literal(lhs), PatternKind::Literal(rhs)) => lhs == rhs,
            (PatternKind::List(lhs, ltail), PatternKind::List(rhs, rtail)) => {
                lhs == rhs && ltail == rtail
            }
            (PatternKind::Record(lhs, ltail), PatternKind::Record(rhs, rtail)) => {
                lhs == rhs && ltail == rtail
            }
            (PatternKind::Any(lhs), PatternKind::Any(rhs)) => Rc::ptr_eq(lhs, rhs),
            (PatternKind::Bound, PatternKind::Bound) => true,
            (PatternKind::Unbound, PatternKind::Unbound) => true,
            (PatternKind::All(lhs), PatternKind::All(rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl Hash for PatternKind {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            PatternKind::Struct(value) => ("struct", value).hash(hasher),
            PatternKind::Variable(value) => ("variable", value).hash(hasher),
            PatternKind::Literal(value) => ("literal", value).hash(hasher),
            PatternKind::List(value, tail) => ("list", value, tail).hash(hasher),
            PatternKind::Record(value, tail) => ("record", value, tail).hash(hasher),
            PatternKind::Any(value) => ("any", Rc::as_ptr(value)).hash(hasher),
            PatternKind::Bound => "bound".hash(hasher),
            PatternKind::Unbound => "unbound".hash(hasher),
            PatternKind::All(patterns) => ("all", patterns).hash(hasher),
        }
    }
}

impl PatternKind {
    pub fn set_age(self, age: usize) -> Self {
        match self {
            PatternKind::Variable(var) => PatternKind::Variable(var.set_current(age)),
            _ => self,
        }
    }

    /// All variables in this pattern, resolved to a particular generation
    pub fn variables<'a>(&'a self, generation: usize) -> Box<dyn Iterator<Item = Variable> + 'a> {
        match self {
            Self::Struct(s) => Box::new(s.variables(generation)),
            Self::Variable(variable) => Box::new(std::iter::once(variable.set_current(generation))),
            Self::List(head, tail) => Box::new(
                head.iter()
                    .flat_map(move |pattern| pattern.variables(generation))
                    .chain(
                        tail.iter()
                            .flat_map(move |pattern| pattern.variables(generation)),
                    ),
            ),
            Self::Record(head, tail) => Box::new(
                head.iter()
                    .flat_map(move |(_, pattern)| pattern.variables(generation))
                    .chain(
                        tail.iter()
                            .flat_map(move |pattern| pattern.variables(generation)),
                    ),
            ),
            Self::All(patterns) => Box::new(
                patterns
                    .iter()
                    .flat_map(move |pattern| pattern.variables(generation)),
            ),
            _ => Box::new(std::iter::empty()),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::List(..) | Self::Record(..))
    }
}

impl Display for PatternKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PatternKind::Literal(lit) => lit.fmt(f),
            PatternKind::List(head, tail) => {
                write!(f, "[")?;
                for (i, pattern) in head.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    pattern.fmt(f)?;
                }
                match tail {
                    Some(tail) => write!(f, ", ..{}]", tail),
                    None => write!(f, "]"),
                }
            }
            PatternKind::Record(head, tail) => {
                write!(f, "{{ ")?;
                for (i, (key, pattern)) in head.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, pattern)?;
                }
                match tail {
                    Some(tail) => write!(f, ", ..{} }}", tail),
                    None => write!(f, " }}"),
                }
            }
            PatternKind::Struct(structure) => structure.fmt(f),
            PatternKind::Any(any) => write!(f, "[{:?}]", Rc::as_ptr(any)),
            PatternKind::Variable(var) => var.fmt(f),
            PatternKind::Bound => "!".fmt(f),
            PatternKind::Unbound => "?".fmt(f),
            PatternKind::All(inner) => {
                for pat in inner {
                    pat.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}

impl From<ast::Pattern> for PatternKind {
    fn from(ast: ast::Pattern) -> PatternKind {
        match ast {
            ast::Pattern::Literal(lit) => Self::Literal(lit),
            ast::Pattern::Variable(id) => {
                Self::Variable(Variable::new_generationless(Identifier::from(id)))
            }
            ast::Pattern::Struct(st) => Self::Struct(Struct::from(st)),
            ast::Pattern::List(list, rest) => Self::List(
                list.into_iter().map(Pattern::from).collect(),
                rest.map(|pat| Pattern::from(*pat)),
            ),
            ast::Pattern::Record(record, rest) => {
                Self::Record(Fields::from(record), rest.map(|pat| Pattern::from(*pat)))
            }
            ast::Pattern::Wildcard => {
                Self::Variable(Variable::new_generationless(Identifier::wildcard("_")))
            }
            ast::Pattern::Bound => Self::Bound,
            ast::Pattern::Unbound => Self::Unbound,
            ast::Pattern::All(patterns) => {
                Self::All(patterns.into_iter().map(Pattern::from).collect())
            }
        }
    }
}
