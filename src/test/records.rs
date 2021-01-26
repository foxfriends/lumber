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
