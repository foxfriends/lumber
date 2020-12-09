use super::*;

test! {
    glob_from_core => r#"
    :- pub(hello/2).
    :- use(@core).

    hello(A, B) :- equal(A, B).
    "#
    ?- "hello(1, A)"
        A = Value::integer(1);
}
