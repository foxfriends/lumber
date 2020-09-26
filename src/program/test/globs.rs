use super::*;

yes! {
    glob_easy => r#"
    :- mod(a).
    :- use(a).
    hello :- atest.
    "#
}

yes! {
    glob_multiple => r#"
    :- mod(a).
    :- mod(b).
    :- use(a).
    :- use(b).
    hello :- atest, btest.
    "#
}

no! {
    glob_undefined => r#"
    :- mod(a).
    :- use(a).
    hello :- atest.
    "#
}

no! {
    glob_private => r#"
    :- mod(a).
    :- use(a).
    hello :- atest.
    "#
}

no! {
    glob_conflict => r#"
    :- mod(a).
    :- mod(b).
    :- use(a).
    :- use(b).
    hello :- test.
    "#
}

yes! {
    glob_conflict_unused => r#"
    :- mod(a).
    :- mod(b).
    :- use(a).
    :- use(b).
    hello :- atest, btest.
    "#
}

yes! {
    glob_conflict_solved => r#"
    :- mod(a).
    :- mod(b).
    :- use(a).
    :- use(b).
    :- use(a(test/0)).
    hello :- test.
    "#
}

yes! {
    glob_redefine => r#"
    :- mod(a).
    :- use(a).
    test.
    "#
}

yes! {
    glob_redefine_and_use => r#"
    :- mod(a).
    :- use(a).
    test :- a::test.
    "#
}

no! {
    glob_recursive => r#"
    :- mod(a).
    :- mod(b).
    "#
}
