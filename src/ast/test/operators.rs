use super::*;

yes! {
    operator_relational_defined => r#"
    :- op(&&, and/2).
    and(true, true).
    test(A, B) :- A && B.
    "#
}

yes! {
    operator_relational_imported => r#"
    :- use(ops(&&)).
    :- mod(ops).
    test(A, B) :- A && B.
    "#
}

no! {
    operator_relational_not_defined => r#"
    and(true, true).
    test(A, B) :- A && B.
    "#
}

no! {
    operator_relational_imported_not_exported => r#"
    :- use(ops(&&)).
    :- mod(ops).
    test(A, B) :- A && B.
    "#
}

no! {
    operator_relational_imported_not_defined => r#"
    :- use(ops(&&)).
    :- mod(ops).
    test(A, B) :- A && B.
    "#
}

no! {
    operator_relational_imported_module_not_defined => r#"
    :- use(ops(&&)).
    test(A, B) :- A && B.
    "#
}

yes! {
    operator_relational_defined_unary => r#"
    :- op(+, positive/1).
    positive(0).
    test(A) :- +A.
    "#
}

no! {
    operator_relational_defined_wrong_arity => r#"
    :- op(&&, and/2).
    and(true, true).
    test(A) :- && A.
    "#
}

no! {
    operator_relational_defined_unary_postfix => r#"
    :- op(+, positive/1).
    positive(0).
    test(A) :- A+.
    "#
}
