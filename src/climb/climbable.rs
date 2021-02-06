use super::Associativity;

pub(crate) trait Climbable {
    fn prec(&self) -> usize;
    fn assoc(&self) -> Associativity;
}

impl<T> Climbable for &T
where
    T: Climbable,
{
    fn prec(&self) -> usize {
        (*self).prec()
    }

    fn assoc(&self) -> Associativity {
        (*self).assoc()
    }
}
