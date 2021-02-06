use super::*;
use crate::ast;
use crate::{List, Record, Struct, Value};
use im_rc::{OrdMap, Vector};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Pattern {
    pattern: Rc<PatternKind>,
    age: Option<usize>,
}

impl Pattern {
    pub fn new(kind: PatternKind, age: usize) -> Self {
        let kind = match kind {
            PatternKind::Variable(var) => PatternKind::Variable(var.set_current(Some(age))),
            _ => kind,
        };
        Self {
            pattern: Rc::new(kind),
            age: Some(age),
        }
    }

    pub fn kind(&self) -> &PatternKind {
        self.pattern.as_ref()
    }

    pub fn age(&self) -> Option<usize> {
        self.age
    }

    pub fn default_age(&self, age: Option<usize>) -> Self {
        if self.age.is_some() || age.is_none() {
            return self.clone();
        }
        match self.pattern.as_ref() {
            p @ PatternKind::Variable(var) if var.generation().is_none() => {
                Self::new(p.clone(), age.unwrap())
            }
            _ => Self {
                pattern: self.pattern.clone(),
                age,
            },
        }
    }

    pub fn variables(&self) -> Box<dyn Iterator<Item = Variable> + '_> {
        let age = self.age;
        Box::new(
            self.pattern
                .variables()
                .map(move |var| var.set_current(age)),
        )
    }

    pub fn record(fields: OrdMap<Atom, Pattern>, rest: Option<Pattern>) -> Self {
        if fields.is_empty() {
            if let Some(rest) = rest {
                return rest;
            }
        }
        Self::from(PatternKind::record(fields, rest))
    }

    pub fn list(items: Vector<Pattern>, tail: Option<Pattern>) -> Self {
        if items.is_empty() {
            if let Some(tail) = tail {
                return tail;
            }
        }
        Self::from(PatternKind::list(items, tail))
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub fn from_value(value: Option<Value>, age: usize) -> Self {
        let kind = match value {
            None => PatternKind::Variable(Variable::new(Identifier::wildcard("_"), age)),
            Some(Value::Integer(int)) => PatternKind::Literal(Literal::Integer(int)),
            Some(Value::Rational(rat)) => PatternKind::Literal(Literal::Rational(rat)),
            Some(Value::String(string)) => PatternKind::Literal(Literal::String(string)),
            Some(Value::List(List { values, complete })) => PatternKind::list(
                values
                    .into_iter()
                    .map(|value| Pattern::from_value(value, age))
                    .collect(),
                if complete {
                    None
                } else {
                    Some(Pattern::new(
                        PatternKind::Variable(Variable::new(Identifier::wildcard("_"), age)),
                        age,
                    ))
                },
            ),
            Some(Value::Record(Record { fields, complete })) => PatternKind::record(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, Pattern::from_value(value, age)))
                    .collect(),
                if complete {
                    None
                } else {
                    Some(Pattern::new(
                        PatternKind::Variable(Variable::new(Identifier::wildcard("_"), age)),
                        age,
                    ))
                },
            ),
            Some(Value::Struct(Struct { name, contents })) => {
                let contents = contents.map(|contents| Pattern::from_value(*contents, age));
                PatternKind::Struct(name, contents)
            }
            Some(Value::Any(any)) => PatternKind::Any(any),
        };
        Pattern::new(kind, age)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.pattern.fmt(f)
    }
}

impl From<ast::Pattern> for Pattern {
    fn from(ast: ast::Pattern) -> Pattern {
        Self::from(PatternKind::from(ast))
    }
}

impl From<PatternKind> for Pattern {
    fn from(kind: PatternKind) -> Self {
        match kind {
            PatternKind::Variable(var) => {
                let age = var.generation();
                Self {
                    pattern: Rc::new(PatternKind::Variable(var)),
                    age,
                }
            }
            _ => Self {
                pattern: Rc::new(kind),
                age: None,
            },
        }
    }
}
