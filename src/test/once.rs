use super::*;

test! {
    once_declarations => r#"
    :- pub(once/2).
    :- pub(multi/2).
    :- use(@core(true/0)).

    once(a, b) ::- true.
    once(a, c) ::- true.
    once(a, d) ::- true.

    multi(a, b) :- true.
    multi(a, c) :- true.
    multi(a, d) :- true.
    "#
    ?- "once(a, B)"
        B = Value::atom("b");
    ?- "multi(a, B)"
        B = Value::atom("b");
        B = Value::atom("c");
        B = Value::atom("d");
}

test! {
    once_branches => r#"
    :- pub(once/2).
    :- pub(multi/2).
    :- use(@core(equal/2)).

    once(A, B) :-
        equal(A, a) ->> equal(B, b);
        equal(A, a) ->> equal(B, c);
        equal(A, a) ->> equal(B, d).

    multi(A, B) :-
        equal(A, a), equal(B, b);
        equal(A, a), equal(B, c);
        equal(A, a), equal(B, d).
    "#
    ?- "once(a, B)"
        B = Value::atom("b");
    ?- "multi(a, B)"
        B = Value::atom("b");
        B = Value::atom("c");
        B = Value::atom("d");
}
