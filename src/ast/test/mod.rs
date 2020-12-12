use crate::Lumber;
use std::path::PathBuf;

macro_rules! yes {
    ($name:ident $(@$lib:ident)* $($handle:literal)* => $src:literal) => {
        #[test]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.join(stringify!($name));
            let result = Lumber::builder()
                $(.bind($handle, |_| unimplemented!()))*
                $(.link(stringify!($lib), Lumber::from_file(here.join("lib").join(stringify!($lib)).join("lib.lumber")).unwrap()))*
                .build(path, $src);
            if let Err(error) = &result {
                eprintln!("{}", error);
                assert!(result.is_ok());
            }
        }
    };
}

macro_rules! no {
    ($name:ident $(@$lib:ident)* $($handle:literal)* => $src:literal) => {
        #[test]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.parent().unwrap().join(stringify!($name));
            assert!(Lumber::builder()
                $(.bind($handle, |_| unimplemented!()))*
                $(.link(stringify!($lib), Lumber::from_file(here.join("lib").join(stringify!($lib)).join("lib.lumber")).unwrap()))*
                .build(path, $src)
                .is_err());
        }
    };
}

mod aliases;
mod comments;
mod definitions;
mod exports;
mod globs;
mod imports;
mod incompletes;
mod libraries;
mod modules;
mod mutables;
mod natives;
mod operations;
mod predicates;
mod values;
mod variables;
