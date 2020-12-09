use super::{Lumber, Value};
use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::path::Path;

/// A builder to customize the construction of a Lumber program.
///
/// To build a non-trivial Lumber program will likely require using this builder.
pub struct LumberBuilder<'p> {
    core: bool,
    test: bool,
    context: Context<'p>,
    natives: HashMap<Handle, NativeFunction<'p>>,
}

impl<'p> LumberBuilder<'p> {
    pub(super) fn new() -> Self {
        Self {
            core: true,
            test: false,
            context: Context::default(),
            natives: HashMap::default(),
        }
    }

    /// Sets whether to include the `@core` module or not. `@core` is included by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Lumber;
    /// Lumber::builder()
    ///     .core(false) // Disables linking to @core
    ///     // ...
    /// #   ;
    /// ```
    pub fn core(mut self, core: bool) -> Self {
        self.core = core;
        self
    }

    /// Sets whether to run the tests or not. Tests will be run when `build` is called,
    /// causing an `Err` to be returned if the tests fail.
    ///
    /// It is recommended that you run the tests as part of your test suit, but build without
    /// tests for release. Tests are not run by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Lumber;
    /// Lumber::builder()
    ///     .test(true) // Runs the tests
    ///     // ...
    /// #   ;
    /// ```
    pub fn test(mut self, test: bool) -> Self {
        self.test = test;
        self
    }

    /// Bind a native function to the Lumber program.
    ///
    /// Arbitrary Rust code can be attached to the Lumber program at a particular
    /// handle. The Lumber program must declare this handle using the `:- nat` directive:
    ///
    /// # Examples
    ///
    /// Anwhere in the Lumber program, a native function can be declared using the `:- nat`
    /// directive. Here, we have defined a module `fmt`, in which the native function `print/1`
    /// is declared.
    ///
    /// ```lumber
    /// // main.lumber
    /// :- mod(fmt).
    /// :- use(fmt(print/1)).
    /// main :- print("Hello World").
    ///
    /// // fmt.lumber
    /// :- nat(print/1). // Here we declare the native function
    /// :- pub(print/1).
    /// ```
    ///
    /// Then, when constructing the Lumber instance, we bind the handle `fmt::print/1` to a
    /// function implemented in Rust.
    ///
    /// ```rust
    /// # use lumber::Lumber;
    /// # use std::path::PathBuf;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let path_to_main = PathBuf::from(file!()).parent().unwrap().join("test/bind/main.lumber");
    /// let lumber = Lumber::builder()
    ///     .bind("fmt::print/1", |_| todo!())
    ///     .build_from_file(path_to_main)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// If the handle is not a valid handle.
    pub fn bind<H, F>(mut self, handle: H, native: F) -> Self
    where
        H: AsHandle,
        F: Fn(Vec<Option<Value>>) -> Box<dyn Iterator<Item = Vec<Option<Value>>>> + 'p, // TODO: this is not the final type
    {
        self.natives.insert(
            handle.as_handle().expect("Invalid handle"),
            NativeFunction::new(native),
        );
        self
    }

    /// Provide a library which may be referenced by this Lumber program.
    ///
    /// Libraries may contain any Lumber definitions, and can even be bound to native
    /// functions (see [`LumberBuilder::bind`][]). Construct a Lumber library the same
    /// way you would construct your main Lumber program.
    ///
    /// Public (`:- pub`) definitions from the provided Lumber program here will be made
    /// available to the program being compiled through a library (`@<name>`) reference
    /// of the same name that is given to the library here.
    ///
    /// # Examples
    ///
    /// Here, we define a Lumber library to perform simple operations on lists, such as
    /// counting its length. The main module then references these functions using a library
    /// reference to `@list`:
    ///
    /// ```lumber
    /// // libs/list.lumber
    /// :- pub(len/2).
    /// len!([]) <- 0.
    /// len!([_, ..R]) <- 1 + len!(R).
    ///
    /// // src/main.lumber
    /// add_lens!(A, B) :- @list::len!(A) + @list::len!(B).
    /// ```
    ///
    /// The library and module are then both compiled and linked together in Rust:
    ///
    /// ```rust
    /// # use lumber::Lumber;
    /// # use std::path::PathBuf;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let path_to_lib_list = PathBuf::from(file!()).parent().unwrap().join("test/link/list.lumber");
    /// # let path_to_main = PathBuf::from(file!()).parent().unwrap().join("test/link/main.lumber");
    /// let lumber = Lumber::builder()
    ///     .link("list", Lumber::from_file(path_to_lib_list)?)
    ///     .build_from_file(path_to_main)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn link<S>(mut self, name: S, program: Lumber<'p>) -> Self
    where
        S: AsRef<str>,
    {
        let (modules, library) = program.into_library(name.as_ref());
        self.context
            .libraries
            .insert(crate::ast::Atom::from(name.as_ref()), library);
        self.context.modules.extend(&mut modules.into_iter());
        self
    }

    /// Uses this builder to compile a source file located on the file system. Without any calls
    /// to other [`LumberBuilder`][] methods, this is the same as calling [`Lumber::from_file`][]
    /// directly.
    ///
    /// # Errors
    ///
    /// If the main module's source file, or the source files of any submodules cannot be found,
    /// an error (typically wrapping an [`std::io::Error`]) is returned.
    ///
    /// If the main module is malformed (due to a syntax or referential error), the program will
    /// fail to parse and a parse error (or multiple parse errors) will be returned. This error
    /// can be shown to the user to hopefully aid in debugging the issue.
    pub fn build_from_file<S>(self, source: S) -> crate::Result<Lumber<'p>>
    where
        S: AsRef<Path>,
    {
        let source_code = std::fs::read_to_string(&source)?;
        self.build(source, source_code)
    }

    /// Uses this builder to compile a Lumber program directly from a source string. Without any
    /// calls to other [`LumberBuilder`][] methods, this is the same as calling [`Lumber::from_str`][]
    /// directly.
    ///
    /// # Errors
    ///
    /// If the source files of any submodules cannot be found, an error (typically wrapping an
    /// [`std::io::Error`]) is returned.
    ///
    /// If the main module is malformed (due to a syntax or referential error), the program will
    /// fail to parse and a parse error (or multiple parse errors) will be returned. This error
    /// can be shown to the user to hopefully aid in debugging the issue.
    pub fn build_from_str<S>(self, source: S) -> crate::Result<Lumber<'p>>
    where
        S: AsRef<str>,
    {
        let source_dir = std::env::current_dir()?;
        self.build(source_dir, source)
    }

    /// Uses this builder to compile a Lumber program from a source string.
    ///
    /// Submodules referenced from the main module are resolved relative to the directory wich is
    /// passed in to this method.
    ///
    /// # Errors
    ///
    /// If the source files of any submodules cannot be found, an error (typically wrapping an
    /// [`std::io::Error`]) is returned.
    ///
    /// If the main module is malformed (due to a syntax or referential error), the program will
    /// fail to parse and a parse error (or multiple parse errors) will be returned. This error
    /// can be shown to the user to hopefully aid in debugging the issue.
    pub fn build<P, S>(mut self, root: P, source: S) -> crate::Result<Lumber<'p>>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        if self.core {
            crate::core::LIB.with(|lib| {
                let (modules, library) = lib.clone().into_library("core");
                self.context
                    .libraries
                    .insert(crate::ast::Atom::from("core"), library);
                self.context.modules.extend(&mut modules.into_iter());
            });
        }
        Lumber::new(self.context, root, source, self.natives, self.test)
    }
}
