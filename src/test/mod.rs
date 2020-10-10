use crate::*;
use std::convert::TryFrom;
use std::path::PathBuf;

macro_rules! test {
    ($name:ident => $src:literal $(?- $query:literal $($($var:ident = $value:expr),*;)*)+) => {
        #[test]
        #[allow(unused_variables)]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.join(stringify!($name));
            let program = Lumber::builder().build(path, $src).unwrap();
            $(
                let question = Question::try_from($query).unwrap();
                let mut answers = program.query::<Binding>(&question);
                $(
                    let answer = question.answer(&answers.next().unwrap().unwrap()).unwrap();
                    $(
                        assert_eq!(answer.get(stringify!($var)).unwrap().as_ref().unwrap(), &$value);
                    )*
                )*
                assert!(answers.next().is_none());
            )+
        }
    };
}

mod accessibility;
mod bindings;
mod operators;
