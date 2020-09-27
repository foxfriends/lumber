use super::Lumber;
use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::path::Path;

pub struct LumberBuilder<'p> {
    core: bool,
    context: Context<'p>,
    natives: HashMap<Handle, NativeFunction<'p>>,
}

impl<'p> LumberBuilder<'p> {
    pub(super) fn new() -> Self {
        Self {
            core: true,
            context: Context::default(),
            natives: HashMap::default(),
        }
    }

    pub fn core(mut self, core: bool) -> Self {
        self.core = core;
        self
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

    pub fn link<S>(mut self, name: S, program: Lumber<'p>) -> Self
    where
        S: AsRef<str>,
    {
        self.context
            .libraries
            .insert(self.context.atomizer.atomize_str(name.as_ref()), program);
        self
    }

    pub fn build_from_file<S>(self, source: S) -> crate::Result<Lumber<'p>>
    where
        S: AsRef<Path>,
    {
        let source_code = std::fs::read_to_string(&source)?;
        self.build(source, source_code)
    }

    pub fn build_from_str<S>(self, source: S) -> crate::Result<Lumber<'p>>
    where
        S: AsRef<str>,
    {
        let source_dir = std::env::current_dir()?;
        self.build(source_dir, source)
    }

    pub fn build<P, S>(mut self, root: P, source: S) -> crate::Result<Lumber<'p>>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        if self.core {
            crate::core::LIB.with(|lib| {
                let core = self.context.atomizer.atomize_str("core");
                self.context.libraries.insert(core, lib.clone());
            });
        }
        Lumber::new(self.context, root, source, self.natives)
    }
}
