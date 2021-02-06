use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub(crate) struct Query {
    /// The shape of this query.
    pub(crate) handle: Handle,
    /// The args in each field.
    pub(crate) args: Vec<Expression>,
}

impl Query {
    pub fn variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.args.iter().flat_map(|pattern| pattern.variables())
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn args(&self) -> &[Expression] {
        &self.args
    }
}

impl AsRef<Handle> for Query {
    fn as_ref(&self) -> &Handle {
        &self.handle
    }
}

impl AsMut<Handle> for Query {
    fn as_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.handle.scope.fmt(f)?;
        if self.args.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        for (i, pattern) in self.args.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            if i < self.handle.arity.len as usize {
                pattern.fmt(f)?;
            } else {
                let mut j = self.handle.arity.len;
                for (name, len) in &self.handle.arity.fields {
                    if i as u32 == j {
                        write!(f, "{}: {}", name, pattern)?;
                        break;
                    }
                    j += len;
                    if (i as u32) < j {
                        pattern.fmt(f)?;
                        break;
                    }
                }
            }
        }
        write!(f, ")")
    }
}

impl From<ast::Query> for Query {
    fn from(ast: ast::Query) -> Self {
        Self {
            handle: ast.handle,
            args: ast.args.into_iter().map(Expression::from).collect(),
        }
    }
}
