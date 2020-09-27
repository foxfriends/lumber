use super::*;

yes! {
    function_literal => r#"
    test! <- 3.
    "#
}

no! {
    function_colon_dash => r#"
    test! :- 3.
    "#
}

yes! {
    function_arguments => r#"
    test!(A, B) <- A + B.
    "#
}

yes! {
    function_assumptions => r#"
    test!(A, B) <-
        C <- A + 2,
        D <- B - 2,
        C * D.
    "#
}

yes! {
    function_aggregate => r#"
    node(_, _, _).
    test!(A, B) <- { pair(X, Y) : node(A, X, _), node(B, _, Y) }.
    "#
}

no! {
    function_unifications => r#"
    test(a).
    test!(A, B) <-
        test(A),
        test(B),
        A + B.
    "#
}

yes! {
    function_call => r#"
    test!(A) <- A + 2.
    test!(A, B) <- test!(A) + test!(B).
    "#
}

// TODO: this will probably be a future feature
no! {
    function_call_nested => r#"
    call!(A) <- A + 2.
    test!(A) <- call!(call!(A)).
    "#
}

no! {
    function_call_undefined => r#"
    test!(A) <- call!(A).
    "#
}
