use super::*;

test! {
    simple_conjunction => r#"
    :- pub(test/2).
    hello(a).
    test(A, B) :- hello(A), hello(B).
    "#
    ?- "test(a, a)";
    ?- "test(a, _)";
    ?- "test(_, _)";
    ?- "test(_, a)";
    ?- "test(A, B)"
        A = Value::atom("a"), B = Value::atom("a");
    ?- "test(A, A)"
        A = Value::atom("a");
    ?- "test(a, b)"
    ?- "test(b, a)"
    ?- "test(_, b)"
    ?- "test(b, _)"
}

test! {
    more_conjunction => r#"
    :- pub(test/2).
    hello(a).
    hello(b).
    test(A, B) :- hello(A), hello(B).
    "#
    ?- "test(a, b)";
    ?- "test(a, a)";
    ?- "test(b, a)";
    ?- "test(b, b)";
    ?- "test(A, B)"
        A = Value::atom("a"), B = Value::atom("a");
        A = Value::atom("a"), B = Value::atom("b");
        A = Value::atom("b"), B = Value::atom("a");
        A = Value::atom("b"), B = Value::atom("b");
    ?- "test(A, A)"
        A = Value::atom("a");
        A = Value::atom("b");
    ?- "test(A, _)"
        A = Value::atom("a");
        A = Value::atom("a");
        A = Value::atom("b");
        A = Value::atom("b");
}

test! {
    conjunction_different_wildcards => r#"
    :- pub(test/1).
    left(a).
    right(b).
    test(C) :-
        left(C),
        right(C).
    "#
    ?- "test(_)"
    ?- "test(a)"
    ?- "test(b)"
}
