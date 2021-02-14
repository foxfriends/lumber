use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) enum Term {
    Expression(Expression),
    PrefixOp(Operator, Box<Term>),
    InfixOp(Box<Term>, Operator, Box<Term>),
    Value(Pattern),
    ListAggregation(Pattern, Body),
}

impl Term {
    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Expression(expression) => expression.handles_mut(),
            Self::Value(..) => Box::new(std::iter::empty()),
            Self::InfixOp(lhs, operator, rhs) => Box::new(
                std::iter::once(operator.handle_mut())
                    .chain(lhs.handles_mut())
                    .chain(rhs.handles_mut()),
            ),
            Self::PrefixOp(operator, term) => {
                Box::new(std::iter::once(operator.handle_mut()).chain(term.handles_mut()))
            }
            Self::ListAggregation(.., body) => Box::new(body.handles_mut()),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Expression(expression) => write!(f, "({})", expression),
            Self::Value(pattern) => pattern.fmt(f),
            Self::InfixOp(lhs, operator, rhs) => write!(f, "{} {} {}", lhs, operator, rhs),
            Self::PrefixOp(operator, term) => write!(f, "{} {}", operator, term),
            Self::ListAggregation(pattern, body) => write!(f, "[{} : {}]", pattern, body),
        }
    }
}

impl From<Pattern> for Term {
    fn from(pattern: Pattern) -> Self {
        Self::Value(pattern)
    }
}

impl From<ast::Term> for Term {
    fn from(ast: ast::Term) -> Self {
        match ast {
            ast::Term::Expression(expr) => Self::Expression(Expression::from(expr)),
            ast::Term::PrefixOp(op, term) => Self::PrefixOp(op, Box::new(Term::from(*term))),
            ast::Term::InfixOp(lhs, op, rhs) => {
                Self::InfixOp(Box::new(Term::from(*lhs)), op, Box::new(Term::from(*rhs)))
            }
            ast::Term::Value(pattern) => Self::Value(Pattern::from(pattern)),
            ast::Term::ListAggregation(pattern, body) => {
                Self::ListAggregation(Pattern::from(pattern), Body::from(body))
            }
        }
    }
}

impl Variables for Term {
    fn variables(&self, vars: &mut Vec<Variable>) {
        match self {
            Self::Expression(expression) => expression.variables(vars),
            Self::Value(pattern) => pattern.variables(vars),
            Self::PrefixOp(.., term) => term.variables(vars),
            Self::InfixOp(lhs, .., rhs) => {
                lhs.variables(vars);
                rhs.variables(vars);
            }
            Self::ListAggregation(pattern, body) => {
                pattern.variables(vars);
                body.variables(vars);
            }
        }
    }
}
