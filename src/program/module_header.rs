use super::*;
use std::collections::{HashMap, HashSet};

/// Lists the predicates and exports of the module, but does not bind them to any
/// actual definitions.
#[derive(Default, Debug)]
pub(crate) struct ModuleHeader {
    /// Modules from which imports are globbed.
    globs: HashSet<Scope>,
    /// Publicly available predicates.
    exports: HashSet<Handle>,
    /// All (private and public) predicates.
    definitions: HashSet<Handle>,
    /// Imported predicates, and their alises.
    aliases: HashMap<Handle, Handle>,
}

impl ModuleHeader {
    pub fn insert_glob(&mut self, module: Scope) -> Option<Scope> {
        self.globs.replace(module)
    }

    pub fn insert_public(&mut self, handle: Handle) -> Option<Handle> {
        self.definitions.insert(handle.clone());
        self.exports.replace(handle)
    }

    pub fn insert(&mut self, handle: Handle) {
        self.definitions.insert(handle);
    }

    pub fn insert_alias(&mut self, alias: Handle, source: Handle) -> Option<(Handle, Handle)> {
        self.aliases
            .insert(alias.clone(), source)
            .map(|source| (alias, source))
    }

    pub fn exports(&self, handle: &Handle) -> bool {
        self.exports.contains(handle)
    }

    pub fn exports_like(&self, handle: &Handle) -> Option<&Handle> {
        self.exports.iter().find(|export| export.like(handle))
    }

    pub fn declares(&self, handle: &Handle) -> bool {
        self.definitions.contains(handle)
    }

    pub fn aliases(&self, handle: &Handle) -> Option<&Handle> {
        self.aliases.get(handle)
    }

    pub fn globbed_modules(&self) -> impl Iterator<Item = &Scope> {
        self.globs.iter()
    }
}
