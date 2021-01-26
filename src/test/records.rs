use super::*;

test! {
    record_unifications => ""
    ?- "@core::equal({:}, {:})";
    ?- "@core::equal({ a: a, b: b }, { a: a, ..B })"
        B = record! { "b" => Value::atom("b") };
    ?- "@core::equal({ a: a, b: b }, { a: a, ..B }), @core::equal({ c: c, ..B }, C)"
        B = record! { "b" => Value::atom("b") },
        C = record! { "c" => Value::atom("c"), "b" => Value::atom("b") };
}

test! {
    record_in_params => "
    :- pub(recordTest/3).
    :- use(@core).

    recordTest(
        T,
        P,
        test { p: P, ..Ts },
    ) :-
        T =:= test { p: A, q: B, .. },
        A < B,
        T =:= test { p: _, ..Ts }.
    "
    ?- "recordTest(test { p: 3, q: 4 }, 5, test { p: 5, q: 4 })";
    ?- "recordTest(test { p: 3, q: 4 }, 5, test(O))"
        O = record! { "p" => 5, "q" => 4 };
}

// This is a weird bug I have experienced, seems like after destructuring G and then passing G
// again to another function, G gets changed and then fails to unify on the way back out. This
// should not be mutating G, everything is supposed to be immutable!
test! {
    record_destructure_and_then_call => "
    :- pub(recordTest/3).

    test(test { c: C, .. }, C).
    hello(1).

    recordTest(
        G,
        P,
        test { a: P, ..Gs },
    ) :-
        G =:= test { a: _, ..Gs },
        test(G, 3).
    "
    ?- "recordTest(test { a: 1, b: 2, c: 3 }, 4, test { a: 4, b: 2, c: 3 })";
}
