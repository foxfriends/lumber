use super::*;
use crate::ast;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Pattern(Rc<PatternKind>);

impl Pattern {
    pub fn new(kind: PatternKind) -> Self {
        Self(Rc::new(kind))
    }

    pub fn wildcard() -> Self {
        Self::new(PatternKind::Variable(Variable::new_generationless(
            Identifier::wildcard("_"),
        )))
    }

    pub fn kind(&self) -> &PatternKind {
        self.0.as_ref()
    }

    pub fn variables(&self, generation: usize) -> Box<dyn Iterator<Item = Variable> + '_> {
        self.0.variables(generation)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ast::Pattern> for Pattern {
    fn from(ast: ast::Pattern) -> Pattern {
        Self::new(PatternKind::from(ast))
    }
}
