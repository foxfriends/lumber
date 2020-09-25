use super::*;

yes! {
    modules_basic => r#"
    :- mod(a).
    "#
}

yes! {
    modules_multiple => r#"
    :- mod(a).
    :- mod(b).
    test(A) :- a::test(A), b::test(A).
    "#
}

yes! {
    modules_reference_parent => r#"
    :- mod(a).
    test(a, b).
    "#
}

no! {
    modules_import_undefined => r#"
    :- mod(a).
    :- use(a(test/2)).
    hello :- test(a, b).
    "#
}

no! {
    modules_import_undefined_unused => r#"
    :- mod(a).
    :- use(a(test/2)).
    "#
}

no! {
    modules_undefined => r#"
    :- mod(a).
    "#
}

no! {
    modules_reference_undefined => r#"
    :- mod(a).
    hello :- a::test(a).
    "#
}
