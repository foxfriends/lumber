use super::*;

test! {
    op_add => r#"
    :- use(@core(+)).
    :- pub(add1/2).
    :- pub(join/3).
    add1(A, B) :- B =:= A + 1.
    join(A, B, C) :- C =:= A + " " + B.
    "#
    ?- "add1(1, A)"
        A = Value::integer(2);
    ?- "add1(3, A)"
        A = Value::integer(4);
    ?- "add1(3.5, A)"
        A = Value::rational(4.5);
    ?- "add1(A, 3)"
    ?- "add1(\"str\", A)"
    ?- "join(\"hello\", \"world\", A)"
        A = Value::string("hello world");
}

test! {
    op_sub => r#"
    :- use(@core(-)).
    :- pub(sub1/2).
    sub1(A, B) :- B =:= A - 1.
    "#
    ?- "sub1(1, A)"
        A = Value::integer(0);
    ?- "sub1(3.5, A)"
        A = Value::rational(2.5);
    ?- "sub1(\"str\", A)"
    ?- "sub1(A, 3)"
}

test! {
    op_mul => r#"
    :- use(@core(*)).
    :- pub(square/2).
    square(A, B) :- B =:= A * A.
    "#
    ?- "square(1, A)"
        A = Value::integer(1);
    ?- "square(3, A)"
        A = Value::integer(9);
    ?- "square(3, A)"
        A = Value::integer(9);
    ?- "square(1.5, A)"
        A = Value::rational(2.25);
    ?- "square(A, 9)"
}

test! {
    op_div => r#"
    :- use(@core(/)).
    :- pub(half/2).
    half(A, B) :- B =:= A / 2.
    "#
    ?- "half(1, A)"
        A = Value::integer(0);
    ?- "half(1.0, A)"
        A = Value::rational(0.5);
    ?- "half(A, 1.5)"
}

test! {
    op_mod => r#"
    :- use(@core(%)).
    :- pub(rem5/2).
    rem5(A, B) :- B =:= A % 5.
    "#
    ?- "rem5(1, A)"
        A = Value::integer(1);
    ?- "rem5(4, A)"
        A = Value::integer(4);
    ?- "rem5(4.0, A)"
    ?- "rem5(A, 5)"
}
