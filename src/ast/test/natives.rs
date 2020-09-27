use super::*;

yes! {
    native_correct "test/2" => r#"
    :- nat(test/2).
    hello :- test(1, 2).
    "#
}

no! {
    native_redefined "test/2" => r#"
    :- nat(test/2).
    test(a, b).
    "#
}

yes! {
    native_exported "test/2" => r#"
    :- nat(test/2).
    :- pub(test/2).
    "#
}

yes! {
    native_imported "a::test/2" => r#"
    :- mod(a).
    :- use(a(test/2)).
    "#
}

no! {
    native_unbound => r#"
    :- nat(test/2).
    "#
}

no! {
    native_twice "test/2" => r#"
    :- nat(test/2).
    :- nat(test/2).
    "#
}

no! {
    native_wrong_module "a::test/2" => r#"
    :- mod(a).
    :- nat(test/2).
    "#
}

yes! {
    native_bound_unused "test/2" => ""
}
