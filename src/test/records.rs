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
