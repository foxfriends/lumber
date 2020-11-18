use super::*;

test! {
    core_equal => r#"
    :- pub(test/2).
    test(A, B) :- @core::equal(A, B).
    "#
    ?- "test(a, a)";
    ?- "test(a, _)";
    ?- "test(a, b)"
}
