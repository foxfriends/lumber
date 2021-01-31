use crate::*;
use std::convert::TryFrom;
use std::path::PathBuf;

macro_rules! test {
    ($name:ident => $src:literal $(?- $query:literal $($($var:ident = $value:expr),*;)*)*) => {
        #[test]
        #[allow(unused_variables, unused_mut)]
        fn $name() {
            let here = PathBuf::from(file!()).parent().unwrap().to_owned();
            let path = here.join(stringify!($name));
            let program = Lumber::builder().test(true).build(path, $src);
            let program = match program {
                Ok(program) => program,
                Err(error) => {
                    eprintln!("{}", error);
                    assert!(false);
                    return;
                },
            };
            $(
                let question = Question::try_from($query).expect("question error");
                let mut answers = program.ask(&question);
                $(
                    let mut answer = answers
                        .next()
                        .expect(&format!("{:?} - expected another answer", $query));
                    $(
                        assert_eq!(
                            answer
                                .remove(stringify!($var))
                                .expect(&format!("{:?}: var {:?} should be set", $query, stringify!($var))),
                            $value,
                            "{:?} -> {} = {}",
                            $query,
                            stringify!($var),
                            $value,
                        );
                    )*
                    assert!(answer.into_iter().map(|(_, value)| value).all(|val| val.is_none()));
                )*
                assert!(answers.next().is_none(), "{:?} expected no more answers", $query);
            )*
        }
    };
}

mod accessibility;
mod bindings;
mod conjunction;
mod disjunction;
mod imports;
mod once;
mod operators;
mod procession;
mod records;
mod relations;
mod tests;
