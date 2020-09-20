use super::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Clone, Debug)]
struct DatabaseEntry<'p> {
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
enum DatabaseDefinition<'p> {
    Static(Definition),
    Mutable(RefCell<Definition>),
    Alias(Handle),
    Native(Option<NativeFunction<'p>>),
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
pub struct Database<'p> {
    definitions: HashMap<Handle, DatabaseEntry<'p>>,
}

impl Database<'_> {
    pub(crate) fn apply_header(&mut self, header: &ModuleHeader) {
        for (output, input) in &header.aliases {
            self.definitions.insert(
                output.clone(),
                DatabaseEntry::new(DatabaseDefinition::Alias(input.clone())),
            );
        }
        for native in &header.natives {
            self.definitions.insert(
                native.clone(),
                DatabaseEntry::new(DatabaseDefinition::Native(None)),
            );
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
}

impl FromIterator<(Handle, Definition)> for Database<'_> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Handle, Definition)>,
    {
        let definitions = iter
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
        Self { definitions }
    }
}
