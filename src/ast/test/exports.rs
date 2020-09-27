use super::*;

yes! {
    export_defined => r#"
    :- pub(test/2).
    test(a, b).
    "#
}

yes! {
    export_order => r#"
    test(a, b).
    :- pub(test/2).
    "#
}

no! {
    export_undefined => r#"
    :- pub(test/2).
    "#
}

no! {
    export_twice => r#"
    :- pub(test/2).
    :- pub(test/2).
    test(a, b).
    "#
}

no! {
    export_similar => r#"
    :- pub(test/1).
    test(a: a).
    test(b: a).
    "#
}

yes! {
    export_reexport => r#"
    :- mod(child).
    :- use(child(test/2)).
    :- pub(test/2).
    "#
}

no! {
    export_incomplete => r#"
    :- pub(test/2).
    :- inc(test/2).
    test(a, b).
    "#
}
