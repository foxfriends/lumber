use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Pattern {
    pattern: Rc<PatternKind>,
    age: Option<usize>,
}

impl Pattern {
    pub fn wildcard() -> Self {
        Self::from(PatternKind::Variable(Variable::new_generationless(
            Identifier::wildcard("_"),
        )))
    }

    pub fn kind(&self) -> &PatternKind {
        self.pattern.as_ref()
    }

    pub fn age(&self) -> Option<usize> {
        self.age
    }

    pub fn default_age(&self, age: usize) -> Self {
        Self {
            pattern: self.pattern.clone(),
            age: self.age.or(Some(age)),
        }
    }

    pub fn variables(&self, generation: usize) -> Box<dyn Iterator<Item = Variable> + '_> {
        self.pattern.variables(generation)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.pattern.fmt(f)
    }
}

impl From<ast::Pattern> for Pattern {
    fn from(ast: ast::Pattern) -> Pattern {
        Self::from(PatternKind::from(ast))
    }
}

impl From<PatternKind> for Pattern {
    fn from(kind: PatternKind) -> Self {
        Self {
            pattern: Rc::new(kind),
            age: None,
        }
    }
}
