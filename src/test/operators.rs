use super::*;

test! {
    operator_expressions => r#"
    :- pub(+).
    :- pub(-).
    :- pub(*).
    :- pub(/).
    :- use(@core(add/3, sub/3, mul/3, div/3)).
    :- op(+, add/3, left, 6).
    :- op(-, sub/3, left, 6).
    :- op(*, mul/3, left, 7).
    :- op(/, div/3, left, 7).
    "#
    ?- "A =:= 6 / 3 + 3 * 2"
        A = Value::integer(8);
    ?- "A =:= 6 / (3 + 3) * 2"
        A = Value::integer(2);
    ?- "A =:= (6 / 3 + 3) * 2"
        A = Value::integer(10);
}

test! {
    operator_expressions_reexported => r#"
    :- pub(+).
    :- pub(-).
    :- pub(*).
    :- pub(/).
    :- use(@core(+, -, *, /)).
    "#
    ?- "A =:= 6 / 3 + 3 * 2"
        A = Value::integer(8);
    ?- "A =:= 6 / (3 + 3) * 2"
        A = Value::integer(2);
    ?- "A =:= (6 / 3 + 3) * 2"
        A = Value::integer(10);
}

test! {
    operator_relations => r#"
    :- pub(<).
    :- use(@core(lt/2)).
    :- op(<, lt/2).
    "#
    ?- "3 < 9";
    ?- "A =:= 9, 3 < A"
        A = Value::integer(9);
}

test! {
    operator_relations_reexported => r#"
    :- pub(<).
    :- use(@core(<)).
    "#
    ?- "3 < 9";
    ?- "A =:= 9, 3 < A"
        A = Value::integer(9);
}
