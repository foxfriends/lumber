use super::*;
use std::collections::HashMap;
use std::path::Path;

pub struct ProgramBuilder<'p> {
    natives: HashMap<Handle, NativeFunction<'p>>,
}

impl<'p> ProgramBuilder<'p> {
    pub fn new() -> Self {
        Self {
            natives: HashMap::default(),
        }
    }

    pub fn bind<F>(mut self, handle: Handle, native: F) -> Self
    where
        F: Fn() + 'p, // TODO: this is not the final type
    {
        self.natives.insert(handle, NativeFunction::new(native));
        self
    }

    pub fn build_from_file<S>(self, source: S) -> crate::Result<Program<'p>>
    where
        S: AsRef<Path>,
    {
        let source_code = std::fs::read_to_string(&source)?;
        Program::new(source, source_code, self.natives)
    }

    pub fn build_from_str<S>(self, source: S) -> crate::Result<Program<'p>>
    where
        S: AsRef<str>,
    {
        let source_dir = std::env::current_dir()?;
        Program::new(source_dir, source, self.natives)
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
        Self::new(source_file, source_code, HashMap::default())
    }

    pub fn from_str<S: AsRef<str>>(source_code: S) -> crate::Result<Self> {
        let source_dir = std::env::current_dir()?;
        Self::new(source_dir, source_code, HashMap::default())
    }

    pub fn builder() -> ProgramBuilder<'p> {
        ProgramBuilder::new()
    }

    fn new<P: AsRef<Path>, S: AsRef<str>>(
        source_file: P,
        source_code: S,
        natives: HashMap<Handle, NativeFunction<'p>>,
    ) -> crate::Result<Self> {
        let source_str = source_code.as_ref();
        Context::compile(source_file.as_ref().to_owned(), source_str, natives)
    }

    pub(crate) fn build(database: Database<'p>) -> Self {
        Self { database }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn program_from_str() {
        Program::from_str(
            r#"// Hello this is the program. Let's go
:- pub(program:from_file/1).
:- pub(program:from_source/1).

program(from_file: F, P) :-
    read_file(F, S),
    program(F, S, P).

program(from_source: S, P) :-
    current_dir(D),
    program(D, S, P).

program(F, S, P) :- todo.
"#,
        )
        .unwrap();
    }
}
