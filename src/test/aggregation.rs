use super::*;

test! {
    aggregate_simple => r#"
    :- use(@core).
    :- pub(test/2).

    check([A, _, _], { a: A, .. }).
    check([_, B, _], { b: B, .. }).
    check([_, _, C], { c: C, .. }).
    test(A, Bs) :-
        C =:= A + 1,
        D =:= A + 2,
        E =:= A + 3,
        Bs =:= [B : check([C, D, E], B)].
    "#
    ?- "test(3, B)"
        B = list![
            record! { "a" => 4 },
            record! { "b" => 5 },
            record! { "c" => 6 },
        ];
}
