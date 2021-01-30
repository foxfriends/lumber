use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

/// A unification against the database, used to build up a rule.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    /// A single query to be unified with the database.
    Query(Query),
    /// A query represented by an operator.
    Relation(Option<Term>, Atom, Term),
    /// An entire sub-rule of unifications to be made.
    Body(Body),
    /// A direcct unification.
    Unification(Expression, Expression),
}

impl Step {
    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Query(query) => Box::new(std::iter::once(query.as_mut())),
            Self::Body(body) => Box::new(body.handles_mut()),
            Self::Relation(lhs, _, rhs) => Box::new(
                lhs.iter_mut()
                    .flat_map(Term::handles_mut)
                    .chain(rhs.handles_mut()),
            ),
            Self::Unification(lhs, rhs) => Box::new(lhs.handles_mut().chain(rhs.handles_mut())),
        }
    }

    pub fn identifiers<'a>(&'a self) -> Box<dyn Iterator<Item = Identifier> + 'a> {
        match self {
            Self::Query(query) => Box::new(query.identifiers()),
            Self::Body(body) => Box::new(body.identifiers()),
            Self::Relation(lhs, _, rhs) => Box::new(
                lhs.iter()
                    .flat_map(Term::identifiers)
                    .chain(rhs.identifiers()),
            ),
            Self::Unification(pattern, expression) => {
                Box::new(pattern.identifiers().chain(expression.identifiers()))
            }
        }
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Query(query) => query.fmt(f),
            Self::Body(body) => write!(f, "({})", body),
            Self::Relation(Some(lhs), operator, rhs) => write!(f, "{} {} {}", lhs, operator, rhs),
            Self::Relation(None, operator, rhs) => write!(f, "{}{}", operator, rhs),
            Self::Unification(lhs, rhs) => write!(f, "{} =:= {}", lhs, rhs),
        }
    }
}

impl From<ast::Step> for Step {
    fn from(ast: ast::Step) -> Self {
        match ast {
            ast::Step::Query(query) => Self::Query(Query::from(query)),
            ast::Step::Body(body) => Self::Body(Body::from(body)),
            ast::Step::Relation(lhs, op, rhs) => {
                Self::Relation(lhs.map(Term::from), op, Term::from(rhs))
            }
            ast::Step::Unification(lhs, rhs) => {
                Self::Unification(Expression::from(lhs), Expression::from(rhs))
            }
        }
    }
}
