use super::*;

yes! {
    predicate_fact => r#"
    test.
    "#
}

yes! {
    predicate_rule => r#"
    yes.
    test :- yes.
    "#
}

yes! {
    predicate_steps => r#"
    a.
    b.
    c.
    test :- a -> b -> c.
    "#
}

no! {
    predicate_steps_undefined => r#"
    a. c.
    test :- a ! b ! c.
    "#
}

yes! {
    predicate_cases => r#"
    a. b. c.
    test :- a ; b ; c.
    "#
}

no! {
    predicate_cases_undefined => r#"
    a. c.
    test :- a ; b ; c.
    "#
}

yes! {
    predicate_terms => r#"
    a. b. c.
    test :- a , b , c.
    "#
}

no! {
    predicate_terms_undefined => r#"
    a. c.
    test :- a ; b ; c.
    "#
}

yes! {
    predicate_conditions => r#"
    a. b. c.
    test :- a -> b -> c.
    "#
}

no! {
    predicate_conditions_undefined => r#"
    a. c.
    test :- a -> b -> c.
    "#
}

yes! {
    predicate_assumptions => r#"
    check(1, 2).
    test :- A <- 1, B <- 2, check(A, B).
    "#
}

yes! {
    predicate_defined_twice_different => r#"
    a. b.
    test(a) :- a.
    test(b) :- b.
    "#
}

yes! {
    predicate_defined_twice_different_boidy => r#"
    a. b.
    test(a) :- a.
    test(a) :- b.
    "#
}

yes! {
    predicate_defined_twice_exact => r#"
    a.
    test(a) :- a.
    test(a) :- a.
    "#
}

yes! {
    predicate_complex => r#"
    a. b. c. e. d("C"). d("E").
    test :- a, b, (c -> A <- "C"; e -> A <- "E"); d(A).
    "#
}

no! {
    predicate_left_arrow => r#"
    test <- 3.
    "#
}

yes! {
    predicate_never => r#"
    test :- !.
    "#
}
