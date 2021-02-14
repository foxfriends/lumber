#![allow(clippy::redundant_allocation)]
use super::*;
use crate::ast;
use im_rc::{OrdMap, Vector};
use std::any::Any;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A pattern against which other patterns can be unified.
#[derive(Clone, Debug)]
pub(crate) enum PatternKind {
    /// A structured pattern (unifies structurally with another query of the same name).
    Struct(Atom, Option<Pattern>),
    /// A single variable (unifies with anything but only once).
    Variable(Variable),
    /// A literal value (unifies only with itself).
    Literal(Literal),
    /// A list of patterns (unifies with a list of the same length where the patterns each
    /// unify in order).
    List(Vector<Pattern>, Option<Pattern>),
    /// A record, containing a set of fields.
    Record(OrdMap<Atom, Pattern>, Option<Pattern>),
    /// An unknown Rust value.
    Any(Rc<Box<dyn Any>>),
    /// A value that must already be bound, at the time of checking (not wildcard)
    Bound,
    /// A value that must already not be bound, at the time of checking (wildcard only)
    Unbound,
    /// A value that must match multiple patterns
    All(Vec<Pattern>),
}

impl PatternKind {
    pub(super) fn record(mut fields: OrdMap<Atom, Pattern>, tail: Option<Pattern>) -> Self {
        match tail.as_ref().map(|pat| pat.kind()) {
            None | Some(PatternKind::Variable(..)) => PatternKind::Record(fields, tail),
            Some(PatternKind::Record(cont, tail)) => {
                fields.extend(cont.clone());
                PatternKind::record(fields, tail.clone())
            }
            // If the tail cannot unify with a record, then there is a problem.
            _ => panic!("illegal construction of record"),
        }
    }

    pub(super) fn list(mut items: Vector<Pattern>, tail: Option<Pattern>) -> Self {
        match tail.as_ref().map(|pat| pat.kind()) {
            None | Some(PatternKind::Variable(..)) => PatternKind::List(items, tail),
            Some(PatternKind::List(cont, tail)) => {
                items.append(cont.clone());
                PatternKind::list(items, tail.clone())
            }
            // If the tail cannot unify with a list, then there is a problem.
            _ => panic!("illegal construction of list"),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::List(..) | Self::Record(..))
    }
}

impl Eq for PatternKind {}
impl PartialEq for PatternKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PatternKind::Struct(lname, lcontents), PatternKind::Struct(rname, rcontents)) => {
                lname == rname && lcontents == rcontents
            }
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
            PatternKind::Struct(name, contents) => ("struct", name, contents).hash(hasher),
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
            PatternKind::Struct(name, None) => write!(f, "{}", name),
            PatternKind::Struct(name, Some(contents)) if contents.kind().is_container() => {
                write!(f, "{} {}", name, contents)
            }
            PatternKind::Struct(name, Some(contents)) => write!(f, "{} ({})", name, contents),
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
            ast::Pattern::Struct(st) => {
                Self::Struct(st.name, st.contents.map(|pat| Pattern::from(*pat)))
            }
            ast::Pattern::List(list, rest) => Self::list(
                list.into_iter().map(Pattern::from).collect(),
                rest.map(|pat| Pattern::from(*pat)),
            ),
            ast::Pattern::Record(record, rest) => Self::record(
                record.into_iter().collect(),
                rest.map(|pat| Pattern::from(*pat)),
            ),
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

impl Variables for PatternKind {
    fn variables(&self, vars: &mut Vec<Variable>) {
        match self {
            Self::Any(..)
            | Self::Unbound
            | Self::Bound
            | Self::Literal(..)
            | Self::Struct(.., None) => {}
            Self::Variable(variable) => vars.push(variable.clone()),
            Self::Struct(.., Some(contents)) => contents.variables(vars),
            Self::List(head, tail) => {
                for pattern in head.iter().chain(tail.iter()) {
                    pattern.variables(vars);
                }
            }
            Self::Record(head, tail) => {
                for pattern in head.values().chain(tail.iter()) {
                    pattern.variables(vars);
                }
            }
            Self::All(patterns) => {
                for pattern in patterns {
                    pattern.variables(vars);
                }
            }
        }
    }
}
