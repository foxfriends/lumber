use super::*;
use std::collections::HashMap;
use std::path::Path;

/// A full Lumber program, ready to have queries run against it.
#[derive(Default, Clone, Debug)]
pub struct Program {
    database: Database,
}

impl Program {
    pub fn from_file<P: AsRef<Path>>(source_file: P) -> crate::Result<Self> {
        let source_code = std::fs::read_to_string(&source_file)?;
        Self::new(source_file, source_code)
    }

    pub fn from_str<S: AsRef<str>>(source_code: S) -> crate::Result<Self> {
        let source_dir = std::env::current_dir()?;
        Self::new(source_dir, source_code)
    }

    fn new<P: AsRef<Path>, S: AsRef<str>>(source_file: P, source_code: S) -> crate::Result<Self> {
        let source_str = source_code.as_ref();
        Context::compile(source_file.as_ref().to_owned(), source_str)
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
