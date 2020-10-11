use super::*;

test! {
    simple_assumption => r#"
    :- pub(hello/3).
    hello(A, B, C) :- C <- A + B.
    "#
    ?- "hello(1, 2, 3)";
    ?- "hello(1, 2, A)"
        A = Value::integer(3);
    ?- "hello(1, A, B)"
}
