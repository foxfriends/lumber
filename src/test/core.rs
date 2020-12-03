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

test! {
    core_list_in => ""
    ?- "@core::list::in(b, [a, b, c])";
    ?- "@core::list::in(d, [])"
    ?- "@core::list::in(d, [a, b, c])"
}

test! {
    core_list_update => ""
    ?- "@core::list::update([a, b, c], c, d, O)"
        O = Value::List(List::new(vec![Some(Value::atom("a")), Some(Value::atom("b")), Some(Value::atom("d"))], true));
    ?- "@core::list::update([c, c, c], c, d, O)"
        O = Value::List(List::new(vec![Some(Value::atom("d")), Some(Value::atom("c")), Some(Value::atom("c"))], true));
    ?- "@core::list::update([{ a: a, b: b }, { a: a, c: c }], { a: a, ..B }, { a: b, ..B }, O)"
        B = Value::Record(Record::default().with("b", Some(Value::atom("b")))),
        O = Value::List(List::new(vec![
            Some(Value::Record(Record::default().with("a", Some(Value::atom("b"))).with("b", Some(Value::atom("b"))))),
            Some(Value::Record(Record::default().with("a", Some(Value::atom("a"))).with("c", Some(Value::atom("c"))))),
        ], true));
}
