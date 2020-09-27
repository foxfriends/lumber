use super::*;

no! {
    library_unlinked => r#"
    test :- @hello::world.
    "#
}

no! {
    library_private @hello => r#"
    test :- @hello::internal.
    "#
}

yes! {
    library_public @hello => r#"
    test :- @hello::world.
    "#
}

yes! {
    library_import @hello => r#"
    :- use(@hello(world/0)).
    test :- world.
    "#
}
