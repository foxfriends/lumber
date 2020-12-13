use super::*;

test! {
    relation_binary_direct => r#"
    :- pub(&&).
    :- op(&&, and/2).
    and(true, true).
    "#
    ?- "(true) && (true)";
    ?- "(true) && (false)"
}

test! {
    relation_binary_indirect => r#"
    :- pub(test/2).
    :- op(&&, and/2).
    and(true, true).
    test(A, B) :- A && B.
    "#
    ?- "test(true, true)";
    ?- "test(true, false)"
}
