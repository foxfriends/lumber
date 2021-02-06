use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) struct Head {
    /// The shape of this head.
    pub(crate) handle: Handle,
    /// The patterns in each field.
    pub(crate) patterns: Vec<Pattern>,
}

impl AsRef<Handle> for Head {
    fn as_ref(&self) -> &Handle {
        &self.handle
    }
}

impl AsMut<Handle> for Head {
    fn as_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }
}

impl Display for Head {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.handle.scope.fmt(f)?;
        if self.patterns.is_empty() {
            return Ok(());
        }
        write!(f, "(")?;
        for (i, pattern) in self.patterns.iter().enumerate() {
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

impl From<ast::Head> for Head {
    fn from(ast: ast::Head) -> Self {
        Self {
            handle: ast.handle,
            patterns: ast.patterns.into_iter().map(Pattern::from).collect(),
        }
    }
}
