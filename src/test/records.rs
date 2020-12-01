use super::*;

test! {
    record_unifications => ""
    ?- "@core::equal({:}, {:})";
    ?- "@core::equal({ a: a, b: b }, { a: a, ..B })"
        B = Value::Record(Record::default().with("b", Some(Value::atom("b"))));
    ?- "@core::equal({ a: a, b: b }, { a: a, ..B }), @core::equal({ c: c, ..B }, C)"
        B = Value::Record(Record::default().with("b", Some(Value::atom("b")))),
        C = Value::Record(Record::default().with("c", Some(Value::atom("c"))).with("b", Some(Value::atom("b"))));
}
