use super::*;
use std::collections::{HashMap, HashSet};

/// Lists the predicates and exports of the module, but does not bind them to any
/// actual definitions.
#[derive(Debug)]
pub(crate) struct ModuleHeader {
    /// The path to this module.
    pub(crate) scope: Scope,
    /// Modules from which imports are globbed.
    pub(crate) globs: HashSet<Scope>,
    /// Native functions bound to this module.
    pub(crate) natives: HashSet<Handle>,
    /// Publicly available predicates.
    pub(crate) exports: HashSet<Handle>,
    /// Predicates that are modifyable at runtime.
    pub(crate) mutables: HashSet<Handle>,
    /// Predicates which are not completely defined in this module.
    pub(crate) incompletes: HashSet<Handle>,
    /// All (private and public) predicates.
    pub(crate) definitions: HashSet<Handle>,
    /// Imported predicates, and their alises.
    pub(crate) aliases: HashMap<Handle, Handle>,
}

impl ModuleHeader {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            globs: Default::default(),
            natives: Default::default(),
            exports: Default::default(),
            mutables: Default::default(),
            incompletes: Default::default(),
            definitions: Default::default(),
            aliases: Default::default(),
        }
    }

    pub fn insert_glob(&mut self, module: Scope) -> Option<Scope> {
        self.globs.replace(module)
    }

    pub fn insert_public(&mut self, handle: Handle) -> Option<Handle> {
        self.exports.replace(handle)
    }

    pub fn insert_native(&mut self, handle: Handle) -> Option<Handle> {
        self.natives.replace(handle)
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

    pub fn resolve<'a>(
        &'a self,
        handle: &'a Handle,
        from_scope: &Scope,
        context: &'a Context,
    ) -> crate::Result<&'a Handle> {
        let resolved = if self.definitions.contains(handle) || self.natives.contains(handle) {
            handle
        } else if let Some(alias) = self.aliases.get(handle) {
            context
                .modules
                .get(&alias.module())
                .unwrap()
                .resolve(alias, &self.scope, context)?
        } else {
            let candidates = self
                .globbed_modules()
                .map(|scope| context.modules.get(scope).unwrap())
                .flat_map(|module| module.resolve_like(handle, from_scope, context))
                .collect::<Vec<_>>();

            match candidates.as_slice() {
                &[] => {
                    return Err(crate::Error::parse(format!(
                        "Unresolved predicate {} in scope {}.",
                        handle, from_scope
                    )))
                }
                &[handle] => handle,
                _ => {
                    return Err(crate::Error::parse(format!(
                        "Ambiguous reference {}. Could be referring to any of:\n{}",
                        handle,
                        candidates
                            .iter()
                            .map(|candidate| format!("\t{}", candidate))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )))
                }
            }
        };

        if self.scope >= *from_scope || self.exports.contains(handle) {
            Ok(resolved)
        } else {
            Err(crate::Error::parse(format!(
                "Predicate {} is not visible from scope {}.",
                handle, from_scope
            )))
        }
    }

    pub fn resolve_like<'a>(
        &'a self,
        handle: &'a Handle,
        from_scope: &Scope,
        context: &'a Context,
    ) -> Vec<&'a Handle> {
        let resolved = if let Some(definition) =
            self.definitions.iter().find(|def| def.like(handle))
        {
            vec![definition]
        } else if let Some((_, alias)) = self.aliases.iter().find(|(alias, _)| alias.like(handle)) {
            let resolved =
                context
                    .modules
                    .get(&alias.module())
                    .unwrap()
                    .resolve(alias, &self.scope, context);
            match resolved {
                Ok(resolved) => vec![resolved],
                Err(..) => vec![],
            }
        } else if let Some(native) = self.natives.iter().find(|native| native.like(handle)) {
            vec![native]
        } else {
            self.globbed_modules()
                .map(|scope| context.modules.get(scope).unwrap())
                .flat_map(|module| module.resolve_like(handle, from_scope, context).into_iter())
                .collect()
        };
        if self.scope >= *from_scope {
            resolved
        } else {
            resolved
                .into_iter()
                .filter(|resolved| self.exports.iter().any(|export| export.like(resolved)))
                .collect()
        }
    }

    pub fn globbed_modules(&self) -> impl Iterator<Item = &Scope> {
        self.globs.iter()
    }

    pub fn errors(&self, context: &Context, native_handles: &[&Handle]) -> Vec<crate::Error> {
        let mut errors = vec![];
        for module in &self.globs {
            if !context.modules.contains_key(module) {
                errors.push(crate::Error::parse(format!(
                    "Unresolved module {} in glob import.",
                    module,
                )));
            }
        }
        for native in &self.natives {
            if !native_handles.contains(&native) {
                errors.push(crate::Error::parse(format!(
                    "Native function {} is not bound.",
                    native,
                )));
            }
            if self.definitions.contains(native) {
                errors.push(crate::Error::parse(format!(
                    "Native function {} cannot also be implemented.",
                    native,
                )));
            } else if self.aliases.contains_key(native) {
                errors.push(crate::Error::parse(format!(
                    "Native function {} cannot also be imported.",
                    native,
                )));
            } else if self.mutables.contains(native) {
                errors.push(crate::Error::parse(format!(
                    "Native function {} cannot be set as mutable.",
                    native,
                )));
            } else if self.incompletes.contains(native) {
                errors.push(crate::Error::parse(format!(
                    "Native function {} cannot be set as incomplete.",
                    native,
                )));
            }
        }
        for export in &self.exports {
            if !self.resolve(export, &self.scope, context).is_ok() {
                errors.push(crate::Error::parse(format!(
                    "Exported predicate {} cannot be found.",
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
        let mut reported: HashSet<Handle> = HashSet::new();
        for alias in self.aliases.values() {
            if reported.contains(alias) {
                continue;
            }
            match context
                .modules
                .get(&alias.module())
                .unwrap()
                .resolve(alias, &self.scope, context)
            {
                Ok(..) => {}
                Err(error) => errors.push(error),
            }
            let aliases = self
                .aliases
                .iter()
                .filter(|&(_, value)| alias == value)
                .map(|(key, _)| key)
                .collect::<Vec<_>>();
            if aliases.len() != 1 {
                reported.insert(alias.clone());
                errors.push(crate::Error::parse(format!(
                    "{} is aliased multiple times, as:\n\t{}",
                    alias,
                    aliases
                        .into_iter()
                        .map(|alias| format!("\t{}", alias))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )));
            }
        }
        errors
    }
}
