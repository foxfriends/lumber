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
        Rest = record! { "b" => 2, "c" => 3 };
    ?- "@core::equal({ a: 1, ..Rest }, { a: 1, b: 2, c: 3 })"
        Rest = record! { "b" => 2, "c" => 3 };
}

test! {
    core_list_in => ""
    ?- "@core::list::in(b, [a, b, c])";
    ?- "@core::list::in(d, [])"
    ?- "@core::list::in(d, [a, b, c])"
}

test! {
    core_list_notin => ""
    ?- "@core::list::notin(b, [a, b, c])"
    ?- "@core::list::notin(d, [])";
    ?- "@core::list::notin(d, [a, b, c])";
}

test! {
    core_list_update => ""
    ?- "@core::list::update([a, b, c], c, d, O)"
        O = list![Value::atom("a"), Value::atom("b"), Value::atom("d")];
    ?- "@core::list::update([c, c, c], c, d, O)"
        O = list![Value::atom("d"), Value::atom("c"), Value::atom("c")];
    ?- "@core::list::update([{ a: a, b: b }, { a: a, c: c }], { a: a, ..B }, { a: b, ..B }, O)"
        B = record! { "b" => Value::atom("b") },
        O = list![
            record! { "a" => Value::atom("b"), "b" => Value::atom("b") },
            record! { "a" => Value::atom("a"), "c" => Value::atom("c") },
        ];
}
