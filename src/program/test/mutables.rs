use super::*;

yes! {
    mutable_basic => r#"
    :- mut(test/1).
    test(a).
    "#
}

yes! {
    mutable_public => r#"
    :- mut(test/1).
    :- pub(test/1).
    test(a).
    "#
}

yes! {
    mutable_import_public => r#"
    :- mod(a).
    :- use(a(test/1)).
    "#
}

no! {
    mutable_import_private => r#"
    :- mod(a).
    :- use(a(test/1)).
    "#
}

no! {
    mutable_alias => r#"
    :- mod(a).
    :- use(a(test/1)).
    :- mut(test/1).
    "#
}

no! {
    mutable_extend => r#"
    :- mod(a).
    :- use(a(test/1)).
    test(b).
    "#
}

yes! {
    mutable_undefined => r#"
    :- mut(test/1).
    "#
}

yes! {
    mutable_import_undefined => r#"
    :- mod(a).
    :- use(a(test/1)).
    "#
}
