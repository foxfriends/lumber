use super::*;

no! {
    op_def_rel_nullary => r#"
    :- op(+, nullary/0).
    nullary.
    "#
}

yes! {
    op_def_rel_unary => r#"
    :- op(+, unary/1).
    unary(a).
    "#
}

yes! {
    op_def_rel_binary => r#"
    :- op(+, binary/2).
    binary(a, b).
    "#
}

no! {
    op_def_rel_ternary => r#"
    :- op(+, ternary/3).
    ternary(a, b, c).
    "#
}

no! {
    op_def_rel_undefined => r#"
    :- op(+, binary/2).
    "#
}

no! {
    op_def_rel_twice => r#"
    :- op(+, binary/2).
    :- op(+, bin/2).
    bin(a, b).
    binary(a, b).
    "#
}

no! {
    op_def_rel_twice_same => r#"
    :- op(+, binary/2).
    :- op(+, binary/2).
    binary(a, b).
    "#
}

yes! {
    op_def_rel_nondet => r#"
    :- op(^, xor/2).
    xor(true, false).
    xor(false, true).
    "#
}
