use super::*;

test! {
    disjunction_simple => r#"
    :- pub(test/2).
    hello(a).
    test(A, B) :- hello(A) ; hello(B).
    "#
    ?- "test(a, b)";
    ?- "test(b, a)";
    ?- "test(c, b)"
    ?- "test(A, b)"
        A = Value::atom("a");
    ?- "test(b, A)"
        A = Value::atom("a");
    ?- "test(A, B)"
        A = Value::atom("a");
}

test! {
    disjunction_multiple_passes => r#"
    :- pub(test/2).
    hello(a).
    hello(b).
    test(A, B) :- hello(A) ; hello(B).
    "#
    ?- "test(a, b)";
    ?- "test(A, B)"
        A = Value::atom("a");
        A = Value::atom("b");
    ?- "test(a, B)";
    ?- "test(c, B)"
        B = Value::atom("a");
        B = Value::atom("b");
    ?- "test(_, B)";;
}
