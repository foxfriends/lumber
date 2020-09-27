use super::*;

yes! {
    import_from_child => r#"
    :- mod(a).
    :- use(a(test/1)).
    "#
}

yes! {
    import_multiple => r#"
    :- mod(a).
    :- use(a(one/1, two/2)).
    "#
}

no! {
    import_same_multiple => r#"
    :- mod(a).
    :- use(a(one/1, one/1)).
    "#
}

no! {
    import_private_from_child => r#"
    :- mod(a).
    :- use(a(test/1)).
    "#
}

yes! {
    import_from_sibling => r#"
    :- mod(a).
    :- mod(b).
    "#
}

no! {
    import_private_from_sibling => r#"
    :- mod(a).
    :- mod(b).
    "#
}

yes! {
    import_from_parent => r#"
    :- mod(a).
    :- pub(test/1).
    test(a).
    "#
}

yes! {
    import_private_from_parent => r#"
    :- mod(a).
    test(a).
    "#
}

yes! {
    import_public_alias_from_parent => r#"
    :- mod(a).
    :- mod(b).
    :- pub(test/1).
    :- use(a(test/1)).
    "#
}

yes! {
    import_private_alias_from_parent => r#"
    :- mod(a).
    :- mod(b).
    :- use(a(test/1)).
    "#
}

yes! {
    import_from_parent_glob => r#"
    :- mod(a).
    :- mod(b).
    :- use(a).
    "#
}

yes! {
    import_from_deep_child => r#"
    :- mod(a).
    :- use(a::b(test/1)).
    "#
}

no! {
    import_private_from_deep_child => r#"
    :- mod(a).
    :- use(a::b(test/1)).
    "#
}

yes! {
    import_from_root => r#"
    :- mod(a).
    test(a).
    "#
}

no! {
    import_from_self_defined => r#"
    :- use(~(test/1)).
    test(a).
    "#
}

no! {
    import_from_self_undefined => r#"
    :- use(~(test/1)).
    "#
}

no! {
    import_recursive => r#"
    :- mod(a).
    :- mod(b).
    "#
}
