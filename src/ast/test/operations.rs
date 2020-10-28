use super::*;

yes! {
    operation_arithmetic => r#"
    test!(A) <- (4 * 3 + 6 / 10) % A.
    "#
}

#[cfg(feature = "builtin-sets")]
yes! {
    operation_sets => r#"
    test1(a).
    test2(b).
    test!(A) <- { _ : test1(A) } && { _ : test2(A) } || { _ }.
    "#
}

no! {
    operation_boolean => r#"
    test!(A, B, C, D) <- A == B && C < D.
    "#
}

yes! {
    operation_named_operator => r#"
    in!(A, [A, ..]) <- [_].
    in!(_, _) <- [].
    test!(A, B) <- A `in` B.
    "#
}

yes! {
    operation_named_operator_alt => r#"
    in(A, [A, ..], [_]).
    in(_, _, []).
    test!(A, B) <- A `in` B.
    "#
}

yes! {
    operation_incorrect_value_types => r#"
    test! <- [] + 3.
    test! <- atom + 3.
    test! <- "hello" / 2.
    "#
}

no! {
    operation_named_operator_wrong_arity => r#"
    in(A, [ A | _ ]).
    in(_, _).
    test!(A, B) <- A `in` B.
    "#
}

no! {
    operation_named_operator_undefined => r#"
    test!(A, B) <- A `in` B.
    "#
}
