use crate::{Struct, Value};
use serde::{ser, Serialize};
use std::collections::HashMap;

/// A serializer to create a Lumber [`Value`][].
pub struct Serializer<'p> {
    output: &'p mut Option<Value>,
}

/// Converts a Rust value to a Lumber value.
pub fn to_value<T>(value: T) -> crate::Result<Value>
where
    T: Serialize,
{
    let mut output = None;
    let mut serializer = Serializer {
        output: &mut output,
    };
    value.serialize(&mut serializer)?;
    Ok(output.unwrap())
}

impl<'a, 'p> ser::Serializer for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> crate::Result<()> {
        *self.output = Some(Value::atom(if value { "true" } else { "false" }));
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> crate::Result<()> {
        *self.output = Some(Value::integer(value));
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> crate::Result<()> {
        *self.output = Some(Value::rational(value));
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> crate::Result<()> {
        *self.output = Some(Value::rational(value));
        Ok(())
    }

    fn serialize_char(self, value: char) -> crate::Result<()> {
        *self.output = Some(Value::string(value));
        Ok(())
    }

    fn serialize_str(self, value: &str) -> crate::Result<()> {
        *self.output = Some(Value::string(value));
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> crate::Result<()> {
        *self.output = Some(Value::from(
            value
                .iter()
                .copied()
                .map(Value::integer)
                .collect::<Vec<_>>(),
        ));
        Ok(())
    }

    fn serialize_none(self) -> crate::Result<()> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> crate::Result<()> {
        Err(crate::Error::ser(
            "unit cannot yet be meaningfully represented by Lumber",
        ))
    }

    fn serialize_unit_struct(self, name: &'static str) -> crate::Result<()> {
        *self.output = Some(Value::atom(name));
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> crate::Result<()> {
        *self.output = Some(Value::Struct(
            Struct::atom(name).with(Some(Value::atom(variant))),
        ));
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        *self.output = Some(Value::Struct(Struct::atom(name).with(output)));
        Ok(())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> crate::Result<()>
    where
        T: Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        *self.output = Some(Value::Struct(
            Struct::atom(name).with(Some(Value::Struct(Struct::atom(variant).with(output)))),
        ));
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> crate::Result<Self::SerializeSeq> {
        *self.output = Some(Value::from(Vec::<Value>::with_capacity(len.unwrap_or(0))));
        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> crate::Result<Self::SerializeTuple> {
        Err(crate::Error::ser(
            "tuples cannot yet be represented by Lumber",
        ))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _index: usize,
    ) -> crate::Result<Self::SerializeTupleStruct> {
        *self.output = Some(Value::atom(name));
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeTupleVariant> {
        *self.output = Some(Value::Struct(
            Struct::atom(name).with(Some(Value::atom(variant))),
        ));
        Ok(self)
    }

    fn serialize_map(self, len: std::option::Option<usize>) -> crate::Result<Self::SerializeMap> {
        *self.output = Some(Value::record(HashMap::with_capacity(len.unwrap_or(0))));
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStruct> {
        *self.output = Some(Value::atom(name));
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStructVariant> {
        *self.output = Some(Value::Struct(
            Struct::atom(name).with(Some(Value::atom(variant))),
        ));
        Ok(self)
    }
}

impl<'a, 'p> ser::SerializeSeq for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        self.output
            .as_mut()
            .unwrap()
            .as_list_mut()
            .unwrap()
            .push(output);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeTuple for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, _value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeTupleStruct for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        self.output
            .as_mut()
            .unwrap()
            .as_struct_mut()
            .unwrap()
            .push(output);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeTupleVariant for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        self.output.as_mut().unwrap().as_struct_mut().unwrap()[0]
            .as_mut()
            .unwrap()
            .as_struct_mut()
            .unwrap()
            .push(output);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeMap for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_key<T>(&mut self, key: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        key.serialize(&mut serializer)?;
        // TODO: we could add some conversions to string from other types here...
        let key_string = output
            .unwrap()
            .as_string()
            .ok_or_else(|| crate::Error::ser("map keys must be strings"))?
            .to_owned();
        self.output
            .as_mut()
            .unwrap()
            .as_record_mut()
            .unwrap()
            .set(key_string, None);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        *self
            .output
            .as_mut()
            .unwrap()
            .as_record_mut()
            .unwrap()
            .iter_mut()
            .filter(|(_, value)| value.is_none())
            .last()
            .unwrap()
            .1 = output;
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> crate::Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let mut key_output = None;
        let mut serializer = Serializer {
            output: &mut key_output,
        };
        key.serialize(&mut serializer)?;
        // TODO: we could add some conversions to string from other types here...
        let key_string = key_output
            .unwrap()
            .as_string()
            .ok_or_else(|| crate::Error::ser("map keys must be strings"))?
            .to_owned();
        let mut value_output = None;
        let mut serializer = Serializer {
            output: &mut value_output,
        };
        value.serialize(&mut serializer)?;
        self.output
            .as_mut()
            .unwrap()
            .as_record_mut()
            .unwrap()
            .set(key_string, value_output);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeStruct for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        self.output
            .as_mut()
            .unwrap()
            .as_struct_mut()
            .unwrap()
            .set(key.to_owned(), vec![output]);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'a, 'p> ser::SerializeStructVariant for &'a mut Serializer<'p> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut output = None;
        let mut serializer = Serializer {
            output: &mut output,
        };
        value.serialize(&mut serializer)?;
        self.output.as_mut().unwrap().as_struct_mut().unwrap()[0]
            .as_mut()
            .unwrap()
            .as_struct_mut()
            .unwrap()
            .set(key.to_owned(), vec![output]);
        Ok(())
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use serde::Serialize;

    #[test]
    fn serialize_integer() {
        assert_eq!(to_value(3u8).unwrap(), Value::integer(3));
        assert_eq!(to_value(3u16).unwrap(), Value::integer(3));
        assert_eq!(to_value(3u32).unwrap(), Value::integer(3));
        assert_eq!(to_value(3u64).unwrap(), Value::integer(3));
        assert_eq!(to_value(3usize).unwrap(), Value::integer(3));
        assert_eq!(to_value(-3i8).unwrap(), Value::integer(-3));
        assert_eq!(to_value(-3i16).unwrap(), Value::integer(-3));
        assert_eq!(to_value(-3i32).unwrap(), Value::integer(-3));
        assert_eq!(to_value(-3i64).unwrap(), Value::integer(-3));
        assert_eq!(to_value(-3isize).unwrap(), Value::integer(-3));
    }

    #[test]
    fn serialize_bool() {
        assert_eq!(to_value(true).unwrap(), Value::atom("true"));
        assert_eq!(to_value(false).unwrap(), Value::atom("false"));
    }

    #[test]
    fn serialize_float() {
        assert_eq!(to_value(3.5f32).unwrap(), Value::rational(3.5));
        assert_eq!(to_value(3.5f64).unwrap(), Value::rational(3.5));
    }

    #[test]
    fn serialize_string() {
        assert_eq!(to_value("Hello").unwrap(), Value::string("Hello"));
        assert_eq!(
            to_value(String::from("Hello")).unwrap(),
            Value::string("Hello")
        );
    }

    #[test]
    fn serialize_char() {
        assert_eq!(to_value('c').unwrap(), Value::string("c"));
    }

    #[test]
    fn serialize_map() {
        let mut map = HashMap::new();
        map.insert("hello", "world");
        map.insert("goodnight", "moon");
        let mut record = Record::default();
        record.set("goodnight", Some(Value::string("moon")));
        record.set("hello", Some(Value::string("world")));
        assert_eq!(to_value(&map).unwrap(), Value::Record(record));
    }

    #[test]
    fn serialize_map_nested() {
        let mut hello_map = HashMap::new();
        hello_map.insert("hello", 5);
        hello_map.insert("bonjour", 7);
        let mut world_map = HashMap::new();
        world_map.insert("england", 7);
        world_map.insert("france", 6);
        let mut map = HashMap::new();
        map.insert("hello", hello_map);
        map.insert("world", world_map);
        let mut hello_record = Record::default();
        hello_record.set("hello", Some(Value::integer(5)));
        hello_record.set("bonjour", Some(Value::integer(7)));
        let mut world_record = Record::default();
        world_record.set("england", Some(Value::integer(7)));
        world_record.set("france", Some(Value::integer(6)));
        let mut record = Record::default();
        record.set("hello", Some(Value::Record(hello_record)));
        record.set("world", Some(Value::Record(world_record)));
        assert_eq!(to_value(&map).unwrap(), Value::Record(record));
    }

    #[test]
    fn serialize_vec() {
        assert_eq!(
            to_value(vec![1, 2]).unwrap(),
            Value::list(vec![Value::integer(1), Value::integer(2)]),
        );
    }

    #[test]
    fn serialize_newtype_struct() {
        #[derive(Serialize)]
        struct NewType(&'static str);
        assert_eq!(
            to_value(NewType("Hello")).unwrap(),
            Value::Struct(Struct::atom("NewType").with(Some(Value::string("Hello")))),
        );
    }

    #[test]
    fn serialize_tuple_struct() {
        #[derive(Serialize)]
        struct Tuple(&'static str, i32);
        assert_eq!(
            to_value(Tuple("Hello", 3)).unwrap(),
            Value::Struct(
                Struct::atom("Tuple")
                    .with(Some(Value::string("Hello")))
                    .with(Some(Value::integer(3)))
            ),
        );
    }

    #[test]
    fn serialize_struct() {
        #[derive(Serialize)]
        struct Test {
            value: &'static str,
            second: i32,
        };
        assert_eq!(
            to_value(Test {
                value: "Hello",
                second: 3
            })
            .unwrap(),
            Value::Struct(
                Struct::atom("Test")
                    .with_entry("value", vec![Some(Value::string("Hello"))])
                    .with_entry("second", vec![Some(Value::integer(3))]),
            ),
        );
    }

    #[test]
    fn serialize_enum_unit() {
        #[derive(Serialize)]
        enum Test {
            Variant,
        }
        assert_eq!(
            to_value(Test::Variant).unwrap(),
            Value::Struct(Struct::atom("Test").with(Some(Value::atom("Variant")))),
        );
    }

    #[test]
    fn serialize_enum_tuple() {
        #[derive(Serialize)]
        enum Test {
            Variant(i32, i32),
        }
        assert_eq!(
            to_value(Test::Variant(1, 2)).unwrap(),
            Value::Struct(
                Struct::atom("Test").with(Some(Value::Struct(
                    Struct::atom("Variant")
                        .with(Some(Value::integer(1)))
                        .with(Some(Value::integer(2)))
                )))
            ),
        );
    }

    #[test]
    fn serialize_enum_struct() {
        #[derive(Serialize)]
        enum Test {
            Variant { first: i32, second: &'static str },
        }
        assert_eq!(
            to_value(Test::Variant {
                first: 1,
                second: "Hello",
            })
            .unwrap(),
            Value::Struct(
                Struct::atom("Test").with(Some(Value::Struct(
                    Struct::atom("Variant")
                        .with_entry("first", vec![Some(Value::integer(1))])
                        .with_entry("second", vec![Some(Value::string("Hello"))])
                )))
            ),
        );
    }
}
