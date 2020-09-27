use super::*;
use std::path::PathBuf;

macro_rules! yes {
    ($name:ident => $src:literal) => {
        #[test]
        fn $name() {
            let path = PathBuf::from(file!());
            let path = path.parent().unwrap().join(stringify!($name));
            Program::builder()
                .build_from_str_with_root(path, $src)
                .unwrap();
        }
    };
}

macro_rules! no {
    ($name:ident => $src:literal) => {
        #[test]
        fn $name() {
            let path = PathBuf::from(file!());
            let path = path.parent().unwrap().join(stringify!($name));
            assert!(Program::builder()
                .build_from_str_with_root(path, $src)
                .is_err());
        }
    };
}

mod aliases;
mod definitions;
mod exports;
mod globs;
mod imports;
mod incompletes;
mod modules;
mod mutables;
mod predicates;
mod values;
mod variables;
