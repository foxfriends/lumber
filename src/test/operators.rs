/* TODO: library calls don't work yet, so this doesn't work either.
use super::*;

test! {
    op_add => r#"
    :- pub(add1/2).
    add1!(A) <- A + 1.
    "#
    ?- "add1(1, A)"
        A = Value::integer(2);
    ?- "add1(3, A)"
        A = Value::integer(4);
    ?- "add1(A, 3)"
}
*/
