use super::*;

test! {
    single_binding => r#"
    :- pub(hello/1).
    hello(1).
    "#
    ?- "hello(A)"
        A = Value::integer(1);
    ?- "hello(B)"
        B = Value::integer(1);
}

test! {
    multiple_binding => r#"
    :- pub(hello/1).
    hello(a).
    hello(b).
    "#
    ?- "hello(A)"
        A = Value::atom("a");
        A = Value::atom("b");
}

test! {
    multiple_variable_single_binding => r#"
    :- pub(hello/2).
    hello(a, b).
    "#
    ?- "hello(A, B)"
        A = Value::atom("a"), B = Value::atom("b");
}

test! {
    multiple_variable_multiple_binding => r#"
    :- pub(hello/2).
    hello(a, b).
    hello(c, d).
    "#
    ?- "hello(A, B)"
        A = Value::atom("a"), B = Value::atom("b");
        A = Value::atom("c"), B = Value::atom("d");
}

test! {
    bind_into_unbound => r#"
    :- pub(hello/2).
    hello(?A, A).
    "#
    ?- "hello(_, 1)";
    ?- "hello(1, 1)"
    ?- "hello(1, B)"
    ?- "hello(A, 1)"
        A = Value::integer(1);
    ?- "hello(A, _)";
}

test! {
    bind_into_bound => r#"
    :- pub(hello/2).
    hello(A, A).
    "#
    ?- "hello(_, !1)"
    ?- "hello(1, !_)";
    ?- "hello(1, !A)"
        A = Value::integer(1);
    ?- "hello(_, !A)"
}

test! {
    bind_complex => r#"
    :- pub(test/3).
    test(A, struct { field: List, .. }, B) :-
        @core::list::in(element { a: A, b: B, c: ?, .. }, List).
    "#
    ?- "test(1, struct { field: [element { a: 1, b: 3, c: 5 }, element { a: 1, b: 2, c: _ }] }, B)"
        B = Value::integer(2);
}
