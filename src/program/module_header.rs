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
    /// Predicates that are modifyable at runtime.
    mutables: HashSet<Handle>,
    /// Predicates which are not completely defined in this module.
    incompletes: HashSet<Handle>,
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
        self.exports.replace(handle)
    }

    pub fn insert_mutable(&mut self, handle: Handle) -> Option<Handle> {
        self.definitions.insert(handle.clone());
        self.mutables.replace(handle)
    }

    pub fn insert_incomplete(&mut self, handle: Handle) -> (Option<Handle>, Option<Handle>) {
        self.definitions.insert(handle.clone());
        (
            self.exports.replace(handle.clone()),
            self.incompletes.replace(handle),
        )
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

    pub fn errors(&self, context: &Context) -> impl Iterator<Item = crate::Error> {
        let mut errors = vec![];
        for module in &self.globs {
            if !context.modules.contains_key(module) {
                errors.push(crate::Error::parse(format!(
                    "Unresolved module {} in glob import.",
                    module,
                )));
            }
        }
        for export in &self.exports {
            if !self.definitions.contains(export) && !self.aliases.contains_key(export) {
                errors.push(crate::Error::parse(format!(
                    "Exported predicate {} has no definition.",
                    export.head(),
                )));
            }
        }
        for mutable in &self.mutables {
            if self.aliases.contains_key(mutable) {
                errors.push(crate::Error::parse(format!(
                    "Cannot set alias {} as mutable.",
                    mutable.head(),
                )));
            }
        }
        for incomplete in &self.incompletes {
            if self.aliases.contains_key(incomplete) {
                errors.push(crate::Error::parse(format!(
                    "Cannot set alias {} as incomplete.",
                    incomplete.head(),
                )));
            }
        }
        for definition in &self.definitions {
            if let Some((key, value)) = self.aliases.get_key_value(definition) {
                let incomplete = context
                    .modules
                    .get(&value.module())
                    .map(|module| module.incompletes.contains(value))
                    .unwrap_or(false);
                if !incomplete {
                    let mut message = format!(
                        "Definition of {} conflicts with imported {}",
                        definition.head(),
                        value,
                    );
                    if !key.like(value) {
                        message.push_str(&format!(" (aliased as {})", key.head()));
                    }
                    message.push('.');
                    errors.push(crate::Error::parse(message));
                }
            }
        }
        errors.into_iter()
    }
}
