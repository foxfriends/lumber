use super::*;

test! {
    simple_query => r#"
    :- pub(hello/0).
    hello.
    "#
    ?- "hello";
}

test! {
    non_exported => r#"
    hello.
    "#
    ?- "hello"
}

test! {
    imported_query => r#"
    :- mod(a).
    "#
    ?- "a::test";
}

test! {
    non_exported_imported_query => r#"
    :- mod(a).
    "#
    ?- "a::test"
}

test! {
    aliased_query => r#"
    :- pub(hello/0).
    :- mod(a).
    :- use(a(alias(test/0, as: hello/0))).
    "#
    ?- "hello";
}

test! {
    non_exported_aliased_query => r#"
    :- mod(a).
    :- use(a(alias(test/0, as: hello/0))).
    "#
    ?- "hello"
}
