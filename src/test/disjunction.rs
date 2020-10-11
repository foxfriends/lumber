use super::*;

test! {
    disjunction_simple => r#"
    :- pub(test/2).
    hello(a).
    test(A, B) :- hello(A) ; hello(B).
    "#
    ?- "test(a, a)";;
    ?- "test(a, b)";
    ?- "test(b, b)"
    ?- "test(b, a)";
    ?- "test(A, b)"
        A = Value::atom("a");
    ?- "test(b, A)"
        A = Value::atom("a");
    ?- "test(A, B)"
        A = Value::atom("a");
        B = Value::atom("a");
}

test! {
    disjunction_multiple_passes => r#"
    :- pub(test/2).
    hello(a).
    hello(b).
    test(A, B) :- hello(A) ; hello(B).
    "#
    ?- "test(a, b)";;
    ?- "test(A, B)"
        A = Value::atom("a");
        A = Value::atom("b");
        B = Value::atom("a");
        B = Value::atom("b");
    ?- "test(a, B)";
        B = Value::atom("a");
        B = Value::atom("b");
    ?- "test(c, B)"
        B = Value::atom("a");
        B = Value::atom("b");
    ?- "test(_, B)"
        ;;
        B = Value::atom("a");
        B = Value::atom("b");
}

test! {
    disjunction_backtracking => r#"
    :- pub(test/2).
    left(a).
    right(b).
    test(A, C) :-
        left(A), C <- a;
        right(A), C <- b.
    "#
    ?- "test(a, A)"
        A = Value::atom("a");
    ?- "test(b, A)"
        A = Value::atom("b");
    ?- "test(c, A)"
    ?- "test(_, A)"
        A = Value::atom("a");
        A = Value::atom("b");
}
