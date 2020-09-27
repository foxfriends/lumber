use super::*;
use std::collections::HashMap;
use std::path::Path;

pub struct ProgramBuilder<'p> {
    context: Context,
    natives: HashMap<Handle, NativeFunction<'p>>,
}

impl<'p> ProgramBuilder<'p> {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
            natives: HashMap::default(),
        }
    }

    pub fn bind<H, F>(mut self, handle: H, native: F) -> crate::Result<Self>
    where
        H: AsHandle,
        F: Fn() + 'p, // TODO: this is not the final type
    {
        self.natives.insert(
            handle.as_handle(&mut self.context)?,
            NativeFunction::new(native),
        );
        Ok(self)
    }

    pub fn build_from_file<S>(self, source: S) -> crate::Result<Program<'p>>
    where
        S: AsRef<Path>,
    {
        let source_code = std::fs::read_to_string(&source)?;
        Program::new(self.context, source, source_code, self.natives)
    }

    pub fn build_from_str<S>(self, source: S) -> crate::Result<Program<'p>>
    where
        S: AsRef<str>,
    {
        let source_dir = std::env::current_dir()?;
        Program::new(self.context, source_dir, source, self.natives)
    }

    pub fn build_from_str_with_root<P, S>(self, root: P, source: S) -> crate::Result<Program<'p>>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        Program::new(self.context, root, source, self.natives)
    }
}

/// A full Lumber program, ready to have queries run against it.
#[derive(Default, Clone, Debug)]
pub struct Program<'p> {
    database: Database<'p>,
}

impl<'p> Program<'p> {
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

    pub fn builder() -> ProgramBuilder<'p> {
        ProgramBuilder::new()
    }

    fn new<P: AsRef<Path>, S: AsRef<str>>(
        context: Context,
        source_file: P,
        source_code: S,
        natives: HashMap<Handle, NativeFunction<'p>>,
    ) -> crate::Result<Self> {
        let source_str = source_code.as_ref();
        context.compile(source_file.as_ref().to_owned(), source_str, natives)
    }

    pub(crate) fn build(database: Database<'p>) -> Self {
        Self { database }
    }
}
