/// Macro to help with creating native functions.
///
/// This will wrap a function definition such that the resulting definition is suitable to
/// be bound to Lumber.
///
/// All parameters to the native function are of type `Option<Value>`, which is omitted from
/// the function signature. The function should not return any value, and instead emits
/// answers using the `answer!` macro. Though it is not checked statically, the answers must
/// be the same length as the parameters, as the emitted values will be bound to the
/// Lumber patterns.
///
/// Under the hood, this uses generators, so using it requires the `generators` and
/// `generator_trait` features to be enabled.
///
/// # Examples
///
/// The definition of the `add` function from the `@core` library, used for the `+` operator.
/// The `add` function returns at most a single answer.
///
/// ```rust
/// #![feature(generators, generator_trait)]
/// use lumber::{native_function, answer};
///
/// native_function! {
///     fn add(lhs, rhs, out) {
///         use lumber::Value::*;
///         match (lhs, rhs, out) {
///             (Some(Integer(lhs)), Some(Integer(rhs)), None)   => answer![lhs, rhs, lhs + rhs],
///             (Some(Integer(lhs)), Some(Rational(rhs)), None)  => answer![lhs, rhs, lhs + rhs],
///             (Some(Rational(lhs)), Some(Integer(rhs)), None)  => answer![lhs, rhs, lhs + rhs],
///             (Some(Rational(lhs)), Some(Rational(rhs)), None) => answer![lhs, rhs, lhs + rhs],
///             (Some(String(lhs)), Some(String(rhs)), None)     => answer![lhs, rhs, lhs + &rhs],
///             _ => {}
///         }
///     }
/// }
/// ```
#[macro_export]
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

/// Construct an answer, to be yielded from a native function. The values passed to this macro
/// are bound to the patterns in Lumber in the same order. Each value must be convertable to a
/// Lumber value using the standard [`Into`][] trait. It is possible to leave an output value
/// unbound using an underscore (`_`) in place of a value.
#[macro_export]
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
