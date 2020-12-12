use super::*;

yes! {
    operation_arithmetic => r#"
    test(A, B) :- B =:= (4 * 3 + 6 / 10) % A.
    "#
}

#[cfg(feature = "builtin-sets")]
yes! {
    operation_sets => r#"
    test1(a).
    test2(b).
    test(A, B) :- B =:= { _ : test1(A) } && { _ : test2(A) } || { _ }.
    "#
}

no! {
    operation_boolean => r#"
    test!(A, B, C, D, E) :- E =:= A == B && C < D.
    "#
}

yes! {
    operation_incorrect_value_types => r#"
    test(A) :- A =:= [] + 3.
    test(A) :- A =:= atom + 3.
    test(A) :- A =:= "hello" / 2.
    "#
}
