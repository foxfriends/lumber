use super::Database;
use crate::ast::Body;
use crate::Binding;

impl Database<'_> {
    pub(crate) fn unify(&self, question: Body) -> impl Iterator<Item = Binding> {
        std::iter::empty()
    }
}
