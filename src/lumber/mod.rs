//! The public API of the Lumber library.
#![deny(missing_docs)]

use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::path::Path;

mod builder;
pub use builder::LumberBuilder;

/// A Lumber program, for use either as a full program, or linked to by another Lumber program
/// as a library.
#[derive(Default, Clone, Debug)]
pub struct Lumber<'p> {
    libraries: HashMap<Atom, Lumber<'p>>,
    database: Database<'p>,
}

impl<'p> Lumber<'p> {
    /// Create a Lumber program where the main module can be found in a file on the file system.
    ///
    /// Submodules referenced from the main module are resolved relative to the same path.
    ///
    /// The resulting program is built linked only to the `@core` library, and with no additional
    /// native functions. If other library linkages and native bindings are required, use the
    /// [`LumberBuilder`][] (created by [`Lumber::builder`][]) instead.
    ///
    /// # Errors
    ///
    /// If the main module's source file, or the source files of any submodules cannot be found,
    /// an error (typically wrapping an [`std::io::Error`]) is returned.
    ///
    /// If the main module is malformed (due to a syntax or referential error), the program will
    /// fail to parse and a parse error (or multiple parse errors) will be returned. This error
    /// can be shown to the user to hopefully aid in debugging the issue.
    pub fn from_file<P: AsRef<Path>>(source_file: P) -> crate::Result<Self> {
        let source_code = std::fs::read_to_string(&source_file)?;
        Self::new(
            Context::with_core(),
            source_file,
            source_code,
            HashMap::default(),
        )
    }

    /// Create a Lumber program from a source string.
    ///
    /// Submodules referenced from the main module are resolved relative to the directory in which
    /// the host program is running (i.e. [`std::env::current_dir`][]).
    ///
    /// The resulting program is built linked only to the `@core` library, and with no additional
    /// native functions. If other library linkages and native bindings are required, use the
    /// [`LumberBuilder`][] (created by [`Lumber::builder`][]) instead.
    ///
    /// # Errors
    ///
    /// If the source files of any submodules cannot be found, an error (typically wrapping an
    /// [`std::io::Error`]) is returned.
    ///
    /// If the main module is malformed (due to a syntax or referential error), the program will
    /// fail to parse and a parse error (or multiple parse errors) will be returned. This error
    /// can be shown to the user to hopefully aid in debugging the issue.
    pub fn from_str<S: AsRef<str>>(source_code: S) -> crate::Result<Self> {
        let source_dir = std::env::current_dir()?;
        Self::new(
            Context::with_core(),
            source_dir,
            source_code,
            HashMap::default(),
        )
    }

    /// Customize the construction of a Lumber program. See [`LumberBuilder`][] for details.
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
