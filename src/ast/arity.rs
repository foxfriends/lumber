use super::*;
use crate::parser::Rule;
use std::fmt::{self, Display, Formatter};

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct Arity {
    pub len: u32,
    pub fields: Vec<(Atom, u32)>,
}

impl Arity {
    pub fn new(pair: crate::Pair) -> Self {
        assert_eq!(pair.as_rule(), Rule::arity);
        let mut pairs = pair.into_inner();
        // TODO: this function will panic if len > 2^32... do we want that?
        let len = pairs.next().unwrap().as_str().parse().unwrap();
        let fields = pairs.fold(vec![], |mut fields, pair| {
            match pair.as_rule() {
                Rule::atom => fields.push((Atom::new(pair), 1)),
                Rule::integer_10 => fields.last_mut().unwrap().1 = pair.as_str().parse().unwrap(),
                _ => unreachable!(),
            }
            fields
        });

        Arity { len, fields }
    }

    pub fn push(&mut self, atom: Atom, len: u32) {
        self.fields.push((atom, len));
    }

    pub fn extend_len(&mut self) {
        *self
            .fields
            .last_mut()
            .map(|field| &mut field.1)
            .unwrap_or(&mut self.len) += 1;
    }

    pub fn new_len(len: u32) -> Self {
        Self {
            len,
            fields: vec![],
        }
    }

    pub fn can_alias(&self, other: &Self) -> bool {
        self.len == other.len
            && self.fields.len() == other.fields.len()
            && self
                .fields
                .iter()
                .zip(other.fields.iter())
                .all(|(a, b)| a.1 == b.1)
    }
}

impl Display for Arity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "/{}", self.len)?;
        for (name, len) in &self.fields {
            write!(f, ":{}", name)?;
            if *len != 1 {
                write!(f, "/{}", len)?;
            }
        }
        Ok(())
    }
}
