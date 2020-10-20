macro_rules! native_function {
    (
        $vis:vis fn $name:ident($($arg_name:ident),+) $body:block
    ) => {
        $vis fn $name(args: Vec<Option<$crate::Value>>) -> Box<dyn Iterator<Item = Vec<Option<$crate::Value>>>> {
            use std::pin::Pin;
            use std::ops::{Generator, GeneratorState};

            let mut args = args.into_iter();
            $( let $arg_name = args.next().unwrap(); )+
            let generator = || $body;

            struct NativeResult<G>
            where G: Unpin + Generator<Yield = Vec<Option<$crate::Value>>, Return = ()> {
                generator: G
            }

            impl<G> Iterator for NativeResult<G>
            where G: Unpin + Generator<Yield = Vec<Option<$crate::Value>>, Return = ()> {
                type Item = Vec<Option<$crate::Value>>;

                fn next(&mut self) -> Option<Self::Item> {
                    match Pin::new(&mut self.generator).resume(()) {
                        GeneratorState::Yielded(output) => Some(output),
                        GeneratorState::Complete(()) => None,
                    }
                }
            }

            Box::new(NativeResult { generator })
        }
    };
}

macro_rules! answer {
    ($($answer:expr),+) => {{ yield answer![@@ $($answer),+]; }};
    (@ $($out:expr,)* @ $val:expr, $($rest:tt)+) => {
        answer![@ $($out,)* Some($val.clone().into()), @ $($rest)+]
    };
    (@ $($out:expr,)* @ _, $($rest:tt)+) => {
        answer![@ $($out,)* None, @ $($rest)+]
    };
    (@ $($out:expr,)* @ $val:expr) => {
        answer![@ $($out,)* Some($val.clone().into()), @]
    };
    (@ $($out:expr,)* @ _) => {
        answer![@ $($out,)* None, @ $($rest)+]
    };
    (@ $($out:expr,)+ @) => { vec![$($out),+] };
}
