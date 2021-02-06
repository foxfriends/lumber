use super::*;

pub(crate) trait Variables<'a> {
    type VarIter: Iterator<Item = Variable> + 'a;

    fn variables(&'a self) -> Self::VarIter;
}
