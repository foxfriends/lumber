use super::*;

yes! {
    value_number => r#"
    test(A) :- A <- 3.
    "#
}

yes! {
    value_string => r#"
    test(A) :- A <- "hello".
    "#
}

yes! {
    value_string_special => r###"
    test(A) :- A <- ##"hey #"# "#" its me"##.
    "###
}

yes! {
    value_atom => r#"
    test(A) :- A <- hello.
    "#
}

yes! {
    value_struct => r#"
    test(A) :- A <- str(a, b).
    "#
}

yes! {
    value_list => r#"
    test(A) :- A <- [1, 2, 3].
    "#
}

yes! {
    value_list_with_empty_tail => r#"
    test(A) :- A <- [1, 2, 3, ..].
    "#
}

yes! {
    value_list_with_tail_pattern => r#"
    test(A) :- A <- [1, 2, 3, ..A].
    "#
}

yes! {
    value_record => r#"
    test(A) :- A <- { a: 1, b: 2, c: 3 }.
    "#
}

yes! {
    value_record_with_empty_tail => r#"
    test(A) :- A <- { a: 1, b: 2, c: 3, .. }.
    "#
}

yes! {
    value_record_with_tail_pattern => r#"
    test(A) :- A <- { a: 1, b: 2, c: 3, ..A }.
    "#
}
