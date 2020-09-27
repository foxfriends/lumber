use super::*;

yes! {
    incomplete_defined => r#"
    :- inc(test/1).
    test(a).
    "#
}

yes! {
    incomplete_not_defined => r#"
    :- inc(test/1).
    "#
}

no! {
    incomplete_and_pub => r#"
    :- inc(test/1).
    :- pub(test/1).
    "#
}

no! {
    incomplete_already_pub => r#"
    :- pub(test/1).
    :- inc(test/1).
    "#
}

yes! {
    incomplete_import_defined => r#"
    :- mod(a).
    :- use(a(test/1)).
    hello :- test(a).
    "#
}

yes! {
    incomplete_import_undefined => r#"
    :- mod(a).
    :- use(a(test/1)).
    hello :- test(a).
    "#
}

yes! {
    incomplete_import_extend => r#"
    :- mod(a).
    :- use(a(test/1)).
    test(b).
    "#
}

no! {
    incomplete_import_redefined => r#"
    :- mod(a).
    :- use(a(test/1)).
    :- inc(test/1).
    "#
}

no! {
    incomplete_extend_scoped => r#"
    :- mod(a).
    a::test(b).
    "#
}

yes! {
    incomplete_import_source => r#"
    :- mod(a).
    :- mod(b).
    test :- a::test(a).
    "#
}

no! {
    incomplete_import_misplaced => r#"
    :- mod(a).
    :- mod(b).
    test :- b::test(a).
    "#
}
