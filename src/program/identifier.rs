/// A unique identifier for a variable.
///
/// Note that the original name of the variable is stored elsewhere, as it is not relevant
/// to the computation but is useful in output and debugging.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Identifier(usize);

impl Identifier {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

impl Into<usize> for Identifier {
    fn into(self) -> usize {
        self.0
    }
}
