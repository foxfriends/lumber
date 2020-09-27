use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::path::Path;

mod builder;
pub use builder::LumberBuilder;

/// A full Lumber program, ready to have queries run against it.
#[derive(Default, Clone, Debug)]
pub struct Lumber<'p> {
    libraries: HashMap<Atom, Lumber<'p>>,
    database: Database<'p>,
}

impl<'p> Lumber<'p> {
    pub fn from_file<P: AsRef<Path>>(source_file: P) -> crate::Result<Self> {
        let source_code = std::fs::read_to_string(&source_file)?;
        Self::new(
            Context::default(),
            source_file,
            source_code,
            HashMap::default(),
        )
    }

    pub fn from_str<S: AsRef<str>>(source_code: S) -> crate::Result<Self> {
        let source_dir = std::env::current_dir()?;
        Self::new(
            Context::default(),
            source_dir,
            source_code,
            HashMap::default(),
        )
    }

    pub fn builder() -> LumberBuilder<'p> {
        LumberBuilder::new()
    }

    fn new<P: AsRef<Path>, S: AsRef<str>>(
        context: Context<'p>,
        source_file: P,
        source_code: S,
        natives: HashMap<Handle, NativeFunction<'p>>,
    ) -> crate::Result<Self> {
        let source_str = source_code.as_ref();
        context.compile(source_file.as_ref().to_owned(), source_str, natives)
    }

    pub(crate) fn build(libraries: HashMap<Atom, Lumber<'p>>, database: Database<'p>) -> Self {
        Self {
            libraries,
            database,
        }
    }

    pub(crate) fn exports(&self, handle: &Handle) -> bool {
        self.database.exports(&handle.without_lib())
    }
}
