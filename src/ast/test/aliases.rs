use super::*;

yes! {
    alias_correct => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    yes(a) :- hello(b).
    "#
}

no! {
    alias_incorrect_arity => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/2))).
    "#
}

no! {
    alias_removed_fields => r#"
    :- mod(a).
    :- use(a(alias(test:from/1:to/1, as: test/4))).
    "#
}

no! {
    alias_moved_fields => r#"
    :- mod(a).
    :- use(a(alias(test:from/1:to/1, as: test:from:to/2))).
    "#
}

yes! {
    alias_redefine_original => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    test(a) :- hello(b).
    "#
}

yes! {
    alias_renamed_fields => r#"
    :- mod(a).
    :- use(a(alias(test:from/1:to/1, as: go:away/1:towards/1))).
    test(a) :- go(away: a, b, towards: c, d).
    "#
}

no! {
    alias_use_original => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    no(a) :- test(a).
    "#
}

no! {
    alias_redefine => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    hello(a) :- here(a).
    "#
}

no! {
    alias_use_scoped => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    no(a) :- a::hello(a).
    "#
}

yes! {
    alias_use_scoped_original => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1))).
    yes(a) :- a::test(a).
    "#
}

no! {
    alias_and_original => r#"
    :- mod(a).
    :- use(a(alias(test/1, as: hello/1), test/1)).
    "#
}

no! {
    alias_backwards => r#"
    :- mod(a).
    :- use(a(alias(hello/1, as: test/1))).
    "#
}

yes! {
    alias_swap => r#"
    :- mod(a).
    :- use(a(alias(hello/1, as: test/1), alias(test/1, as: hello/1))).
    "#
}
