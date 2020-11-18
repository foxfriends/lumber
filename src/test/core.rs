use super::*;

test! {
    core_equal => r#"
    :- pub(test/2).
    test(A, B) :- @core::equal(A, B).
    "#
    ?- "test(a, a)";
    ?- "test(a, _)";
    ?- "test(a, b)"
    ?- "test(a, B)"
        B = Value::atom("a");
    ?- "@core::equal({ a: 1, b: 2, c: 3 }, { a: 1, ..Rest })"
        Rest = Value::Record(Record::default().with("b", Some(Value::integer(2))).with("c", Some(Value::integer(3))));
    ?- "@core::equal({ a: 1, ..Rest }, { a: 1, b: 2, c: 3 })"
        Rest = Value::Record(Record::default().with("b", Some(Value::integer(2))).with("c", Some(Value::integer(3))));
}
