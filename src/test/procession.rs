use super::*;

test! {
    simple_procession => r#"
    :- pub(test/2).
    hello(a).
    hello(b).
    test(A, B) :- hello(A) -> hello(B).
    "#
    ?- "test(a, a)";
    ?- "test(a, b)";
    ?- "test(b, a)";
    ?- "test(b, b)";
    ?- "test(A, b)"
        A = Value::atom("a");
    ?- "test(a, A)"
        A = Value::atom("a");
        A = Value::atom("b");
    ?- "test(A, B)"
        A = Value::atom("a"), B = Value::atom("a");
        A = Value::atom("a"), B = Value::atom("b");
    ?- "test(_, _)";;
}
