use super::*;
use std::collections::HashSet;

/// Lists the predicates and exports of the module, but does not bind them to any
/// actual definitions.
#[derive(Default, Debug)]
pub(crate) struct ModuleHeader {
    /// Publicly available predicates.
    pubs: HashSet<Handle>,
    /// All (private and public) predicates.
    defs: HashSet<Handle>,
}

impl ModuleHeader {
    pub fn insert_public(&mut self, handle: Handle) -> Option<Handle> {
        self.defs.insert(handle.clone());
        self.pubs.replace(handle)
    }

    pub fn insert(&mut self, handle: Handle) {
        self.defs.insert(handle);
    }
}
