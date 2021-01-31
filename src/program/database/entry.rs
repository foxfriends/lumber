use super::*;

#[derive(Clone, Debug)]
pub(crate) struct DatabaseEntry<'p> {
    pub public: bool,
    pub definition: DatabaseDefinition<'p>,
}

impl<'p> DatabaseEntry<'p> {
    pub fn new(definition: DatabaseDefinition<'p>) -> Self {
        Self {
            public: false,
            definition,
        }
    }

    pub fn set_public(&mut self) {
        self.public = true;
    }
}
