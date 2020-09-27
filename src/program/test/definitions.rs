use super::*;

yes! {
    definition_facts => r#"
    animal(cat).
    animal(dog).
    animal(fish).
    dry(cat, always).
    dry(dog, sometimes).
    dry(fish, never).
    "#
}

yes! {
    definition_rules => r#"
    female(evelyn).
    female(dayna).
    male(cameron).
    male(brian).
    parent(evelyn, cameron).
    parent(evelyn, dayna).
    parent(brian, cameron).
    parent(brian, dayna).
    sibling(A, B) :- parent(A, C), parent(B, C).
    mother(A, B) :- female(A), parent(A, B).
    father(A, B) :- male(A), parent(A, B).
    sister(A, B) :- female(A), sibling(A, B).
    brother(A, B) :- male(A), sibling(A, B).
    "#
}

no! {
    definition_undefined_reference => r#"
    test(a) :- yes.
    "#
}

yes! {
    definition_named_fields => r#"
    hello(from: a, to: b).
    test :- hello(from: a, to: b).
    "#
}

no! {
    definition_named_fields_omitted => r#"
    hello(from: a, to: b).
    test :- hello(a, b).
    "#
}
