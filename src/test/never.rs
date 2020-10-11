use super::*;

test! {
    never_basic => r#"
    :- pub(test/0).
    test :- !.
    "#
    ?- "test"
}

test! {
    never_conjunction => r#"
    :- pub(test/0).
    true.
    test :- true, !.
    "#
    ?- "test"
}

test! {
    never_disjunction => r#"
    :- pub(test1/0).
    :- pub(test2/0).
    true.
    test1 :- true; !.
    test2 :- !; true.
    "#
    ?- "test1";
    ?- "test2";
}
