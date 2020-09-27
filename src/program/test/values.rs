use super::*;

yes! {
    value_number => r#"
    test(A) :- A <- 3.
    "#
}

yes! {
    value_string => r#"
    test(A) :- A <- "helo".
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
