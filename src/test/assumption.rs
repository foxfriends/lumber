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

test! {
    backwards_assumption => r#"
    :- pub(hello/3).
    test(1, 2, 3).
    hello(A, B, C) :- C <- test!(A, B).
    "#
    ?- "hello(1, 2, 3)";
    ?- "hello(A, 2, 3)"
        A = Value::integer(1);
    ?- "hello(1, B, 3)"
        B = Value::integer(2);
    ?- "hello(1, 2, C)"
        C = Value::integer(3);
}

test! {
    destructuring_assumption => r#"
    :- pub(hello/3).
    make_test!(A, B) <- test(A, B, 2).
    hello(A, B, C) :-
        test(_, _, C) <- make_test!(A, B).
    "#
    ?- "hello(1, 2, C)"
        C = Value::integer(2);
}
