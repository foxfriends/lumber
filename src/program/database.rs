use super::*;
use crate::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) struct DatabaseEntry<'p> {
    public: bool,
    definition: DatabaseDefinition<'p>,
}

impl<'p> DatabaseEntry<'p> {
    fn new(definition: DatabaseDefinition<'p>) -> Self {
        Self {
            public: false,
            definition,
        }
    }

    fn set_public(&mut self) {
        self.public = true;
    }
}

#[derive(Clone, Debug)]
pub(crate) enum DatabaseDefinition<'p> {
    Static(Definition),
    Mutable(RefCell<Definition>),
    Alias(Handle),
    Native(NativeFunction<'p>),
}

impl DatabaseDefinition<'_> {
    fn set_mutable(&mut self) {
        match self {
            Self::Static(def) => *self = Self::Mutable(RefCell::new(std::mem::take(def))),
            _ => panic!("Cannot change definition to mutable"),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Database<'p> {
    /// All currently active definitions in this program. They may not be the same as they
    /// were when the program was created, due to mutable definitions.
    pub(super) definitions: HashMap<Handle, DatabaseEntry<'p>>,
}

impl<'p> Database<'p> {
    pub fn new<I: IntoIterator<Item = (Handle, Definition)>>(
        definitions: I,
    ) -> Self {
        let definitions = definitions
            .into_iter()
            .fold(
                HashMap::<Handle, Vec<Definition>>::default(),
                |mut handles, (handle, entry)| {
                    handles.entry(handle).or_default().push(entry);
                    handles
                },
            )
            .into_iter()
            .map(|(handle, definition)| {
                (
                    handle,
                    DatabaseEntry::new(DatabaseDefinition::Static(
                        definition.into_iter().collect(),
                    )),
                )
            })
            .collect();
        Self {
            definitions,
        }
    }

    pub fn apply_header(
        &mut self,
        header: &ModuleHeader,
        natives: &HashMap<Handle, NativeFunction<'p>>,
    ) {
        for (output, input) in &header.aliases {
            self.definitions.insert(
                output.clone(),
                DatabaseEntry::new(DatabaseDefinition::Alias(input.clone())),
            );
        }
        for native in &header.natives {
            self.definitions.insert(
                native.clone(),
                DatabaseEntry::new(DatabaseDefinition::Native(
                    natives.get(native).unwrap().clone(),
                )),
            );
        }
        for incomplete in &header.incompletes {
            self.definitions
                .entry(incomplete.clone())
                .or_insert_with(|| {
                    DatabaseEntry::new(DatabaseDefinition::Static(Definition::default()))
                });
        }
        for mutable in &header.mutables {
            self.definitions.entry(mutable.clone()).or_insert_with(|| {
                DatabaseEntry::new(DatabaseDefinition::Static(Definition::default()))
            });
        }
        for export in &header.exports {
            self.definitions.get_mut(export).unwrap().set_public();
        }
        for handle in &header.mutables {
            self.definitions
                .get_mut(handle)
                .unwrap()
                .definition
                .set_mutable();
        }
    }

    pub fn lookup(&self, handle: &Handle, public: bool) -> Option<&DatabaseDefinition<'p>> {
        let entry = self.definitions.get(handle)?;
        if public && !entry.public {
            return None;
        }
        match &entry.definition {
            DatabaseDefinition::Alias(handle) => self.lookup(handle, false),
            definition => Some(definition),
        }
    }

    pub fn exports(&self, handle: &Handle) -> bool {
        self.definitions
            .get(handle)
            .map(|entry| entry.public)
            .unwrap_or(false)
    }
}
