use crate::*;
use std::convert::TryFrom;
use std::path::PathBuf;

macro_rules! test {
    ($name:ident => $src:literal $(?- $query:literal $($($var:ident = $value:expr),*;)*)+) => {
        #[test]
        #[allow(unused_variables, unused_mut)]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.join(stringify!($name));
            let program = Lumber::builder().build(path, $src).expect("syntax error");
            $(
                let question = Question::try_from($query).expect("question error");
                let mut answers = program.query::<Binding>(&question);
                $(
                    let mut answer = question
                        .answer(
                            &answers
                                .next()
                                .expect(&format!("{:?} - expected another answer", $query))
                                .unwrap()
                        )
                        .expect(&format!("{:?} answers should be bound", $query));
                    $(
                        assert_eq!(
                            answer
                                .remove(stringify!($var))
                                .expect(&format!("{:?}: var {:?} should exist", $query, stringify!($var)))
                                .as_ref()
                                .expect(&format!("{:?}: var {:?} should be set", $query, stringify!($var))),
                            &$value,
                            "{:?} -> {} = {}",
                            $query,
                            stringify!($var),
                            $value,
                        );
                    )*
                    assert!(answer.values().all(Option::is_none));
                )*
                assert!(answers.next().is_none(), "{:?} expected no more answers", $query);
            )+
        }
    };
}

mod accessibility;
mod assumption;
mod bindings;
mod conjunction;
mod disjunction;
mod never;
mod operators;
mod procession;
