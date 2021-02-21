use crate::{Error, Record, Struct, Value};
use serde::de::{self, Deserialize, DeserializeSeed, IntoDeserializer, Visitor};
use std::ops::Index;

pub struct Deserializer<'de> {
    path: Vec<String>,
    input: Option<&'de Value>,
}

impl<'de> Deserializer<'de> {
    pub fn from_value(input: &'de Value) -> Self {
        Self::from_value_inner(vec![], input)
    }

    fn from_value_inner(path: Vec<String>, input: &'de Value) -> Self {
        Self {
            path,
            input: Some(input),
        }
    }

    pub fn from_optional(input: Option<&'de Value>) -> Self {
        Self::from_optional_inner(vec![], input)
    }

    fn from_optional_inner(path: Vec<String>, input: Option<&'de Value>) -> Self {
        Self { path, input }
    }
}

pub fn from_value<'de, T>(value: &'de Value) -> crate::Result<T>
where
    T: Deserialize<'de>,
{
    let deserializer = Deserializer::from_value(value);
    let output = T::deserialize(&deserializer)?;
    Ok(output)
}

macro_rules! deserialize_int {
    ($name:ident, $visit:ident) => {
        fn $name<V>(self, visitor: V) -> crate::Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let int = self
                .input
                .and_then(Value::as_integer)
                .ok_or_else(|| self.error("expected integer"))?;
            visitor.$visit(int.into())
        }
    };
}

macro_rules! deserialize_rat {
    ($name:ident, $visit:ident, $t:ty) => {
        fn $name<V>(self, visitor: V) -> crate::Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let rat = self
                .input
                .and_then(Value::as_rational)
                .ok_or_else(|| self.error("expected rational"))?;
            visitor.$visit(rat.to_f64() as $t)
        }
    };
}

impl<'de> Deserializer<'de> {
    fn error(&self, msg: impl std::fmt::Display) -> Error {
        Error::de(msg, &self.path)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = match self.input {
            None => return visitor.visit_none(),
            Some(value) => value,
        };
        match value {
            Value::Integer(..) => self.deserialize_i64(visitor),
            Value::Rational(..) => self.deserialize_f64(visitor),
            Value::String(..) => self.deserialize_string(visitor),
            Value::List(..) => self.deserialize_seq(visitor),
            Value::Struct(st) if st.is_atom() => match st.as_atom().unwrap() {
                "true" | "false" => self.deserialize_bool(visitor),
                _ => Err(self.error("cannot deserialize arbitrary structs")),
            },
            Value::Struct(..) => Err(self.error("cannot deserialize arbitrary structs")),
            Value::Record(..) => self.deserialize_map(visitor),
            Value::Any(..) => Err(self.error("cannot deserialize an `Any` value")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let b = self
            .input
            .and_then(Value::as_struct)
            .and_then(Struct::as_atom)
            .ok_or_else(|| self.error("expected boolean"))?;
        match b {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            _ => Err(self.error("expected boolean")),
        }
    }

    deserialize_int!(deserialize_i8, visit_i8);
    deserialize_int!(deserialize_i16, visit_i16);
    deserialize_int!(deserialize_i32, visit_i32);
    deserialize_int!(deserialize_i64, visit_i64);
    deserialize_int!(deserialize_i128, visit_i128);
    deserialize_int!(deserialize_u8, visit_u8);
    deserialize_int!(deserialize_u16, visit_u16);
    deserialize_int!(deserialize_u32, visit_u32);
    deserialize_int!(deserialize_u64, visit_u64);
    deserialize_int!(deserialize_u128, visit_u128);

    deserialize_rat!(deserialize_f32, visit_f32, f32);
    deserialize_rat!(deserialize_f64, visit_f64, f64);

    fn deserialize_char<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut chars = self
            .input
            .and_then(Value::as_string)
            .ok_or_else(|| self.error("expected string"))?
            .chars();
        let result = visitor.visit_char(
            chars
                .next()
                .ok_or_else(|| self.error("expected non-empty string"))?,
        );
        if chars.next().is_some() {
            return Err(self.error("expected a single character"));
        }
        result
    }

    fn deserialize_str<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self
            .input
            .and_then(Value::as_string)
            .ok_or_else(|| self.error("expected string"))?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(self.error("not implemented: cannot deserialize bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(self.error("not implemented: cannot deserialize byte_buf"))
    }

    fn deserialize_option<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            None => visitor.visit_none(),
            Some(..) => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // TODO: this might be wrong... but it's ok for now.
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self
            .input
            .and_then(Value::as_struct)
            .and_then(Struct::as_atom)
            .ok_or_else(|| self.error("expected atom"))?;
        if atom != name {
            return Err(self.error(format!("expected {} found {}", name, atom)));
        }
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let st = self
            .input
            .and_then(Value::as_struct)
            .ok_or_else(|| self.error("expected struct"))?;
        if st.name() != name {
            return Err(self.error(format!("expected {} found {}", name, st.name())));
        }
        match st.contents() {
            None => Err(self.error("expected a newtype wrapper around a variant struct")),
            Some(contents) => visitor.visit_newtype_struct(&Deserializer::from_optional_inner(
                self.path.clone(),
                contents.as_ref(),
            )),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let list = self
            .input
            .and_then(Value::as_list)
            .ok_or_else(|| self.error("expected list"))?;
        visitor.visit_seq(&mut SeqDeserializer {
            path: self.path.clone(),
            input: list,
            index: 0,
            max: list.len(),
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let list = self
            .input
            .and_then(Value::as_list)
            .ok_or_else(|| self.error("expected list"))?;
        if list.len() != len {
            return Err(self.error(format!(
                "expected a tuple of length {}, got {}",
                len,
                list.len()
            )));
        }
        visitor.visit_seq(&mut SeqDeserializer {
            path: self.path.clone(),
            input: list,
            index: 0,
            max: len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let st = self
            .input
            .and_then(Value::as_struct)
            .ok_or_else(|| self.error("expected struct"))?;
        if st.name() != name {
            return Err(self.error(format!("expected {} found {}", name, st.name())));
        }
        match st.contents() {
            Some(Some(contents))
                if contents.as_list().map(|list| list.len()).unwrap_or(0) == len =>
            {
                visitor.visit_seq(&mut SeqDeserializer {
                    path: self.path.clone(),
                    input: contents.as_list().unwrap(),
                    index: 0,
                    max: len,
                })
            }
            _ => Err(self.error(format!("expected a tuple struct of length {}", len))),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let record = self
            .input
            .and_then(Value::as_record)
            .ok_or_else(|| self.error("expected record"))?;
        visitor.visit_map(&mut MapDeserializer {
            path: self.path.clone(),
            input: record,
            index: 0,
        })
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let st = self
            .input
            .and_then(Value::as_struct)
            .ok_or_else(|| self.error("expected struct"))?;
        if st.name() != name {
            return Err(self.error(format!("expected {} found {}", name, st.name())));
        }
        match st.contents() {
            Some(Some(contents)) if contents.is_record() => {
                let record = contents.as_record().unwrap();
                if !record.iter().all(|(key, _)| fields.contains(&key)) {
                    return Err(self.error("fields of serialized value do not match struct"));
                }
                visitor.visit_map(&mut MapDeserializer {
                    path: self.path.clone(),
                    input: record,
                    index: 0,
                })
            }
            _ => Err(self.error("expected a struct containing a record")),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let st = self
            .input
            .and_then(Value::as_struct)
            .ok_or_else(|| self.error("expected struct"))?;
        if st.name() != name {
            return Err(self.error(format!("expected {} found {}", name, st.name())));
        }
        match st.contents() {
            Some(Some(value)) => {
                let variant_value = value
                    .as_struct()
                    .ok_or_else(|| self.error("expected a variant struct"))?;
                visitor.visit_enum(&mut EnumDeserializer {
                    path: self.path.clone(),
                    input: variant_value,
                })
            }
            _ => Err(self.error("expected a newtype wrapper around a variant struct")),
        }
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("what is an identifier, we don't need this?")
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct SeqDeserializer<'de, I>
where
    I: Index<usize, Output = Option<Value>>,
{
    path: Vec<String>,
    input: &'de I,
    index: usize,
    max: usize,
}

impl<'de, 'a, I> de::SeqAccess<'de> for &'a mut SeqDeserializer<'de, I>
where
    I: Index<usize, Output = Option<Value>>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> crate::Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index == self.max {
            Ok(None)
        } else {
            let mut path = self.path.clone();
            path.push(self.index.to_string());
            let output = seed.deserialize(&Deserializer::from_optional_inner(
                path,
                self.input[self.index].as_ref(),
            ))?;
            self.index += 1;
            Ok(Some(output))
        }
    }
}

struct MapDeserializer<'de> {
    path: Vec<String>,
    input: &'de Record,
    index: usize,
}

impl<'de, 'a> de::MapAccess<'de> for &'a mut MapDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> crate::Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        self.input
            .iter()
            .nth(self.index)
            .map(|(key, _)| seed.deserialize(key.into_deserializer()))
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> crate::Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let element = self
            .input
            .iter()
            .nth(self.index)
            .map(|(key, value)| {
                let mut path = self.path.clone();
                path.push(key.to_owned());
                seed.deserialize(&Deserializer::from_optional_inner(path, value.as_ref()))
            })
            .unwrap();
        self.index += 1;
        element
    }
}

struct EnumDeserializer<'de> {
    path: Vec<String>,
    input: &'de Struct,
}

impl<'de> EnumDeserializer<'de> {
    fn error(&self, msg: impl std::fmt::Display) -> Error {
        Error::de(msg, &self.path)
    }
}

impl<'de, 'a> de::EnumAccess<'de> for &'a mut EnumDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> crate::Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(IntoDeserializer::<Error>::into_deserializer(
            self.input.name(),
        ))?;
        Ok((val, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for &'a mut EnumDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> crate::Result<()> {
        if self.input.is_atom() {
            Ok(())
        } else {
            Err(self.error("expected an atom"))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> crate::Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.input.contents() {
            None => Err(self.error("expected a newtype struct")),
            Some(value) => seed.deserialize(&Deserializer::from_optional_inner(
                self.path.clone(),
                value.as_ref(),
            )),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input.contents() {
            Some(Some(contents))
                if contents.as_list().map(|list| list.len()).unwrap_or(0) == len =>
            {
                visitor.visit_seq(&mut SeqDeserializer {
                    path: self.path.clone(),
                    input: contents.as_list().unwrap(),
                    index: 0,
                    max: len,
                })
            }
            _ => Err(self.error(format!("expected a tuple struct of length {}", len))),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input.contents() {
            Some(Some(contents)) if contents.is_record() => {
                visitor.visit_map(&mut MapDeserializer {
                    path: self.path.clone(),
                    input: contents.as_record().unwrap(),
                    index: 0,
                })
            }
            _ => Err(self.error("expected a struct containing a record")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn deserialize_integer() {
        assert_eq!(from_value::<u8>(&Value::integer(16)).unwrap(), 16u8);
        assert_eq!(from_value::<u16>(&Value::integer(16)).unwrap(), 16u16);
        assert_eq!(from_value::<u32>(&Value::integer(16)).unwrap(), 16u32);
        assert_eq!(from_value::<u64>(&Value::integer(16)).unwrap(), 16u64);
        assert_eq!(from_value::<usize>(&Value::integer(16)).unwrap(), 16usize);
        assert_eq!(from_value::<i8>(&Value::integer(16)).unwrap(), 16i8);
        assert_eq!(from_value::<i16>(&Value::integer(16)).unwrap(), 16i16);
        assert_eq!(from_value::<i32>(&Value::integer(16)).unwrap(), 16i32);
        assert_eq!(from_value::<i64>(&Value::integer(16)).unwrap(), 16i64);
        assert_eq!(from_value::<isize>(&Value::integer(16)).unwrap(), 16isize);
    }

    #[test]
    fn deserialize_rational() {
        assert_eq!(from_value::<f32>(&Value::rational(1.5)).unwrap(), 1.5f32);
        assert_eq!(from_value::<f64>(&Value::rational(1.5)).unwrap(), 1.5f64);
    }

    #[test]
    fn deserialize_string() {
        assert_eq!(
            from_value::<&str>(&Value::string("Hello")).unwrap(),
            "Hello"
        );
        assert_eq!(
            from_value::<String>(&Value::string("Hello")).unwrap(),
            String::from("Hello")
        );
    }

    #[test]
    fn deserialize_bool() {
        assert_eq!(from_value::<bool>(&Value::atom("true")).unwrap(), true);
        assert_eq!(from_value::<bool>(&Value::atom("false")).unwrap(), false);
        assert!(from_value::<bool>(&Value::atom("what")).is_err());
        assert!(from_value::<bool>(&Value::integer(3)).is_err());
    }

    #[test]
    fn deserialize_char() {
        assert_eq!(from_value::<char>(&Value::string("c")).unwrap(), 'c');
        assert!(from_value::<char>(&Value::string("hello")).is_err());
        assert!(from_value::<char>(&Value::string("")).is_err());
    }

    #[test]
    fn deserialize_newtype_struct() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct NewType(bool);

        assert_eq!(
            from_value::<NewType>(&Value::Struct(Struct::new(
                "NewType",
                Some(Value::atom("true"))
            )))
            .unwrap(),
            NewType(true),
        );
        assert!(from_value::<NewType>(&Value::Struct(Struct::new("NewType", None))).is_err());
    }

    #[test]
    fn deserialize_option() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct OptionWrapper(Option<bool>);

        assert_eq!(
            from_value::<OptionWrapper>(&Value::Struct(Struct::new(
                "OptionWrapper",
                Some(Value::atom("true"))
            )))
            .unwrap(),
            OptionWrapper(Some(true)),
        );
        assert_eq!(
            from_value::<OptionWrapper>(&Value::Struct(Struct::new("OptionWrapper", None)))
                .unwrap(),
            OptionWrapper(None),
        );
    }

    #[test]
    fn deserialize_list() {
        assert_eq!(
            from_value::<Vec<&str>>(&list!["a", "b", "c"]).unwrap(),
            vec!["a", "b", "c"],
        );
    }

    #[test]
    fn deserialize_tuple() {
        assert_eq!(
            from_value::<(&str, &str, &str)>(&list!["a", "b", "c"]).unwrap(),
            ("a", "b", "c"),
        );

        assert!(from_value::<(&str, &str)>(&list!["a", "b", "c"]).is_err());
    }

    #[test]
    fn deserialize_tuple_struct() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct Tuple(i32, u32);

        assert_eq!(
            from_value::<Tuple>(&Value::Struct(Struct::new("Tuple", Some(list![-3, 3])))).unwrap(),
            Tuple(-3, 3),
        );

        assert!(
            from_value::<Tuple>(&Value::Struct(Struct::new("NotTuple", Some(list![-3, 3]))))
                .is_err()
        );

        assert!(from_value::<Tuple>(&Value::Struct(Struct::new(
            "Tuple",
            Some(Value::integer(-3))
        )))
        .is_err());

        assert!(from_value::<Tuple>(&Value::Struct(Struct::new(
            "Tuple",
            Some(list![-3, 3, "hi"])
        )))
        .is_err());
    }

    #[test]
    fn deserialize_struct() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct TestStruct {
            first: i32,
            second: u32,
        }

        assert_eq!(
            from_value::<TestStruct>(&Value::Struct(Struct::new(
                "TestStruct",
                Some(record! {
                    "first" => -3,
                    "second" => 3,
                })
            )))
            .unwrap(),
            TestStruct {
                first: -3,
                second: 3
            },
        );

        assert!(from_value::<TestStruct>(&Value::Struct(Struct::new(
            "NotStruct",
            Some(record! {
                "first" => -3,
                "second" => 3,
            })
        )))
        .is_err());

        assert!(from_value::<TestStruct>(&Value::Struct(Struct::new(
            "TestStruct",
            Some(record! {
                "first" => -3,
            })
        )))
        .is_err());

        assert!(from_value::<TestStruct>(&Value::Struct(Struct::new(
            "TestStruct",
            Some(record! {
                "first" => -3,
                "third" => 3,
            })
        )))
        .is_err());

        assert!(from_value::<TestStruct>(&Value::Struct(Struct::new(
            "TestStruct",
            Some(record! {
                "first" => -3,
                "second" => 3,
                "third" => 3,
            })
        )))
        .is_err());
    }

    #[test]
    fn deserialize_unit_struct() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        struct TestUnit;

        assert_eq!(
            from_value::<TestUnit>(&Value::atom("TestUnit")).unwrap(),
            TestUnit,
        );

        assert!(from_value::<TestUnit>(&Value::atom("NotUnit")).is_err());
    }

    #[test]
    fn deserialize_unit_enum() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Enum {
            First,
            Second,
        }

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::atom("First"))
            )))
            .unwrap(),
            Enum::First,
        );

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::atom("Second"))
            )))
            .unwrap(),
            Enum::Second,
        );

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::atom("Third"))
        )))
        .is_err());

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "NotEnum",
            Some(Value::atom("First"))
        )))
        .is_err());

        assert!(from_value::<Enum>(&Value::atom("Enum")).is_err());
        assert!(from_value::<Enum>(&Value::atom("First")).is_err());
    }

    #[test]
    fn deserialize_newtype_enum() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Enum {
            First(String),
            Second(String),
        }

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::Struct(Struct::new(
                    "First",
                    Some(Value::string("Hello"))
                )))
            )))
            .unwrap(),
            Enum::First(String::from("Hello")),
        );

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::Struct(Struct::new(
                    "Second",
                    Some(Value::string("Hello"))
                )))
            )))
            .unwrap(),
            Enum::Second(String::from("Hello")),
        );

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::Struct(Struct::new(
                "Third",
                Some(Value::string("Hello"))
            )))
        )))
        .is_err(),);

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::atom("First"))
        )))
        .is_err());

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "NotEnum",
            Some(Value::Struct(Struct::new(
                "Second",
                Some(Value::string("Hello"))
            )))
        )))
        .is_err(),);

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::Struct(Struct::new(
                "Second",
                Some(list!["Hello", "World"])
            )))
        )))
        .is_err());
    }

    #[test]
    fn deserialize_tuple_enum() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Enum {
            First(String, String),
        }

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::Struct(Struct::new(
                    "First",
                    Some(list!["Hello", "World"])
                )))
            )))
            .unwrap(),
            Enum::First(String::from("Hello"), String::from("World")),
        );

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::Struct(Struct::new(
                "First",
                Some(Value::string("World"))
            )))
        )))
        .is_err());

        assert!(from_value::<Enum>(&Value::Struct(Struct::new(
            "Enum",
            Some(Value::Struct(Struct::new(
                "First",
                Some(list!["World", "World", "World"]),
            )),)
        )))
        .is_err());
    }

    #[test]
    fn deserialize_fields_enum() {
        #[serde(crate = "libserde")]
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Enum {
            First { a: String, b: String },
        }

        assert_eq!(
            from_value::<Enum>(&Value::Struct(Struct::new(
                "Enum",
                Some(Value::Struct(Struct::new(
                    "First",
                    Some(record! {
                        "a" => "Hello",
                        "b" => "World",
                    })
                )))
            )))
            .unwrap(),
            Enum::First {
                a: String::from("Hello"),
                b: String::from("World")
            },
        );
    }

    #[test]
    fn deserialize_record() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(String::from("a"), 1);
        map.insert(String::from("b"), 2);
        let mut record = Record::default();
        record.set("a", Some(Value::integer(1)));
        record.set("b", Some(Value::integer(2)));

        assert_eq!(
            from_value::<HashMap<String, u8>>(&record! {
                "a" => 1,
                "b" => 2,
            })
            .unwrap(),
            map,
        );
    }
}
