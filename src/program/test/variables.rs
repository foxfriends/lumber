use super::*;

yes! {
    variable_used => r#"
    check(yes).
    test(A) :- check(A).
    "#
}

no! {
    variable_singleton_head => r#"
    check(yes).
    test(A) :- check(yes).
    "#
}

yes! {
    variable_wildcard_head => r#"
    check(yes).
    test(_) :- check(yes).
    "#
}

yes! {
    variable_discarded_head => r#"
    check(yes).
    test(_A) :- check(yes).
    "#
}

yes! {
    variable_discarded_used => r#"
    check(yes).
    test(_A) :- check(_A).
    "#
}

yes! {
    variable_in_struct => r#"
    check(yes).
    test(pair(A, B)) :- check(A), check(B).
    "#
}

no! {
    variable_in_struct_singleton => r#"
    check(yes).
    test(pair(A, B)) :- check(A).
    "#
}

yes! {
    variable_internal => r#"
    check(yes).
    compare(yes, yes).
    test(A) :-
        check(B),
        check(A),
        compare(A, B).
    "#
}

yes! {
    variable_discarded_internal => r#"
    check(yes).
    compare(yes, yes).
    test(A) :-
        check(A),
        compare(A, _B).
    "#
}

yes! {
    variable_discarded_internal_used => r#"
    check(yes).
    compare(yes, yes).
    test(A) :-
        check(_B),
        check(A),
        compare(A, _B).
    "#
}
