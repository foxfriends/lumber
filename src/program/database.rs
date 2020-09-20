use super::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Clone, Debug)]
struct DatabaseEntry {
    public: bool,
    definition: DatabaseDefinition,
}

#[derive(Clone, Debug)]
enum DatabaseDefinition {
    Static(Definition),
    Mutable(RefCell<Definition>),
    Alias(Handle),
    Native(NativeFunction),
}

#[derive(Clone, Default, Debug)]
pub struct Database {
    definitions: HashMap<Handle, DatabaseEntry>,
}

impl FromIterator<(Handle, Definition)> for Database {
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
                    DatabaseEntry {
                        public: false,
                        definition: DatabaseDefinition::Static(definition.into_iter().collect()),
                    },
                )
            })
            .collect();
        Self { definitions }
    }
}
