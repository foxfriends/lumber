use super::*;

test! {
    once_declarations => r#"
    :- pub(once/2).
    :- pub(multi/2).
    :- use(@core(true/0)).

    once(a, b) ::- true.
    once(a, c) ::- true.
    once(a, d) ::- true.
    once(b, bb) ::- true.
    once(b, cc) ::- true.
    once(b, dd) ::- true.

    multi(a, b) :- true.
    multi(a, c) :- true.
    multi(a, d) :- true.
    multi(b, bb) :- true.
    multi(b, cc) :- true.
    multi(b, dd) :- true.
    "#
    ?- "once(a, B)"
        B = Value::atom("b");
    ?- "once(b, B)"
        B = Value::atom("bb");
    ?- "multi(a, B)"
        B = Value::atom("b");
        B = Value::atom("c");
        B = Value::atom("d");
    ?- "multi(b, B)"
        B = Value::atom("bb");
        B = Value::atom("cc");
        B = Value::atom("dd");
}

test! {
    once_branches => r#"
    :- pub(once/2).
    :- pub(multi/2).
    :- use(@core(equal/2)).

    once(A, B) :-
        equal(A, a) ->> equal(B, b);
        equal(A, a) ->> equal(B, c);
        equal(A, a) ->> equal(B, d);
        equal(A, b) ->> equal(B, bb);
        equal(A, b) ->> equal(B, cc);
        equal(A, b) ->> equal(B, dd).

    multi(A, B) :-
        equal(A, a), equal(B, b);
        equal(A, a), equal(B, c);
        equal(A, a), equal(B, d);
        equal(A, b), equal(B, bb);
        equal(A, b), equal(B, cc);
        equal(A, b), equal(B, dd).
    "#
    ?- "once(a, B)"
        B = Value::atom("b");
    ?- "once(b, B)"
        B = Value::atom("bb");
    ?- "multi(a, B)"
        B = Value::atom("b");
        B = Value::atom("c");
        B = Value::atom("d");
    ?- "multi(b, B)"
        B = Value::atom("bb");
        B = Value::atom("cc");
        B = Value::atom("dd");
}

test! {
    once_nested => r#"
    :- pub(once/4).
    :- use(@core(equal/2)).

    once(A, B, C, D) :-
        C <- A + B, (
            equal(A, C) ->> equal(D, zero);
            equal(D, B)
        ).
    "#
    ?- "once(1, 1, C, D)"
        C = Value::integer(2), D = Value::integer(1);
    ?- "once(2, 0, C, D)"
        C = Value::integer(2), D = Value::atom("zero");
}
