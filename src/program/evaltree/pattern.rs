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
    pub fn new(kind: PatternKind, age: usize) -> Self {
        let kind = match kind {
            PatternKind::Variable(var) => PatternKind::Variable(var.set_current(Some(age))),
            _ => kind,
        };
        Self {
            pattern: Rc::new(kind),
            age: Some(age),
        }
    }

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

    pub fn default_age(&self, age: Option<usize>) -> Self {
        if self.age.is_some() || age.is_none() {
            return self.clone();
        }
        match self.pattern.as_ref() {
            p @ PatternKind::Variable(var) if var.generation().is_none() => {
                Self::new(p.clone(), age.unwrap())
            }
            _ => Self {
                pattern: self.pattern.clone(),
                age,
            },
        }
    }

    pub fn variables(&self) -> Box<dyn Iterator<Item = Variable> + '_> {
        self.pattern.variables()
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
        match kind {
            PatternKind::Variable(var) => {
                let age = var.generation();
                Self {
                    pattern: Rc::new(PatternKind::Variable(var)),
                    age,
                }
            }
            _ => Self {
                pattern: Rc::new(kind),
                age: None,
            },
        }
    }
}
