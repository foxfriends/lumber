#[doc(hidden)]
#[macro_export]
macro_rules! __list {
    ($els:ident @ $(,)?) => {
        $crate::Value::List($crate::List::new($els))
    };
    ($els:ident @ _) => {{
        $els.push(None);
        $crate::__list!($els @ )
    }};
    ($els:ident @ _, $($rest:tt)*) => {{
        $els.push(None);
        $crate::__list!($els @ $($rest)*)
    }};
    ($els:ident @ $item:expr) => {{
        $els.push(Some($crate::Value::from($item)));
        $crate::__list!($els @ )
    }};
    ($els:ident @ $item:expr, $($rest:tt)*) => {{
        $els.push(Some($crate::Value::from($item)));
        $crate::__list!($els @ $($rest)*)
    }};
}

/// Construct a Lumber list, similarly to constructing a `Vec` using `vec!`, but the `_` can be
/// used in place of a value to insert an unbound element.
///
/// # Examples
///
/// ```rust
/// # use lumber::{Value, List, list};
/// let list = list![1, 2, _, 4];
/// assert_eq!(
///     list,
///     Value::List(List::new(vec![Some(Value::from(1)), Some(Value::from(2)), None, Some(Value::from(4))])),
/// );
/// ```
#[macro_export]
macro_rules! list {
    ($($src:tt)*) => ({
        #[allow(unused_mut)]
        let mut list = vec![];
        $crate::__list!(list @ $($src)*)
    });
}

#[cfg(test)]
mod test_list {
    use crate::*;

    #[test]
    fn empty() {
        assert_eq!(list![], Value::List(List::default()));
    }

    #[test]
    fn not_empty() {
        assert_eq!(
            list![1, 2, 3],
            Value::List(List::new(vec![
                Some(Value::from(1)),
                Some(Value::from(2)),
                Some(Value::from(3))
            ]))
        );
    }

    #[test]
    fn wildcards() {
        assert_eq!(list![_, _], Value::List(List::new(vec![None, None])));
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __record {
    ($els:ident @ $(,)?) => {
        $crate::Value::Record($crate::Record::new($els))
    };
    ($els:ident @ $key:expr => _) => {{
        $els.insert($key.to_owned(), None);
        $crate::__record!($els @ )
    }};
    ($els:ident @ $key:expr => _, $($rest:tt)*) => {{
        $els.insert($key.to_owned(), None);
        $crate::__record!($els @ $($rest)*)
    }};
    ($els:ident @ $key:expr => $item:expr) => {{
        $els.insert($key.to_owned(), Some($crate::Value::from($item)));
        $crate::__record!($els @ )
    }};
    ($els:ident @ $key:expr => $item:expr, $($rest:tt)*) => {{
        $els.insert($key.to_owned(), Some($crate::Value::from($item)));
        $crate::__record!($els @ $($rest)*)
    }};
}

/// Construct a Lumber record using more literal syntax. The `_` can be used in place
/// of a value to insert an unbound element.
///
/// # Examples
///
/// ```rust
/// # use lumber::{Value, Record, record};
/// # use std::collections::HashMap;
/// let record = record!{
///     "a" => 123,
///     "b" => "hello",
///     "c" => _,
/// };
/// let mut hashmap = HashMap::new();
/// hashmap.insert("a".to_owned(), Some(Value::from(123)));
/// hashmap.insert("b".to_owned(), Some(Value::from("hello")));
/// hashmap.insert("c".to_owned(), None);
/// assert_eq!(
///     record,
///     Value::Record(Record::new(hashmap)),
/// );
/// ```
#[macro_export]
macro_rules! record {
    ($($src:tt)*) => ({
        #[allow(unused_mut)]
        let mut record = std::collections::HashMap::new();
        $crate::__record!(record @ $($src)*)
    });
}

#[cfg(test)]
mod test_record {
    use crate::*;

    #[test]
    fn empty() {
        assert_eq!(record! {}, Value::Record(Record::default()));
    }
}

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
        answer![@ $($out,)* None, @]
    };
    (@ $($out:expr,)+ @) => { vec![$($out),+] };
    ($($answer:tt)*) => {{ yield answer![@@ $($answer)*]; }};
}
