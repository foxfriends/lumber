use super::*;
use std::path::PathBuf;

macro_rules! yes {
    ($name:ident $(@$lib:ident)* $($handle:literal)* => $src:literal) => {
        #[test]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.join(stringify!($name));
            Program::builder()
                $(.bind($handle, || {}).unwrap())*
                $(.link(stringify!($lib), Program::from_file(here.join("lib").join(stringify!($lib)).join("lib.lumber")).unwrap()))*
                .build(path, $src)
                .unwrap();
        }
    };
}

macro_rules! no {
    ($name:ident $(@$lib:ident)* $($handle:literal)* => $src:literal) => {
        #[test]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.parent().unwrap().join(stringify!($name));
            assert!(Program::builder()
                $(.bind($handle, || {}).unwrap())*
                $(.link(stringify!($lib), Program::from_file(here.join("lib").join(stringify!($lib)).join("lib.lumber")).unwrap()))*
                .build(path, $src)
                .is_err());
        }
    };
}

mod aliases;
mod comments;
mod definitions;
mod exports;
mod functions;
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
