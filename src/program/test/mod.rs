use super::*;
use std::path::PathBuf;

macro_rules! yes {
    ($name:ident $($handle:literal :- $func:expr),* => $src:literal) => {
        #[test]
        fn $name() {
            let path = PathBuf::from(file!());
            let path = path.parent().unwrap().join(stringify!($name));
            Program::builder()
                $(.bind($handle, $func).unwrap())*
                .build_from_str_with_root(path, $src)
                .unwrap();
        }
    };
}

macro_rules! no {
    ($name:ident $($handle:literal :- $func:expr),* => $src:literal) => {
        #[test]
        fn $name() {
            let path = PathBuf::from(file!());
            let path = path.parent().unwrap().join(stringify!($name));
            assert!(Program::builder()
                $(.bind($handle, $func).unwrap())*
                .build_from_str_with_root(path, $src)
                .is_err());
        }
    };
}

mod aliases;
mod definitions;
mod exports;
mod functions;
mod globs;
mod imports;
mod incompletes;
mod modules;
mod mutables;
mod natives;
mod operations;
mod predicates;
mod values;
mod variables;
