use super::Value;
use crate::ast::*;
use std::collections::HashMap;
use std::iter::FromIterator;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Default, Clone, Debug)]
pub struct Binding(HashMap<Identifier, Pattern>);

impl Binding {
    pub(crate) fn get(&self, identifier: Identifier) -> &Pattern {
        let pattern = self.0.get(&identifier).unwrap();
        match pattern {
            Pattern::Variable(identifier) => self.get(*identifier),
            _ => pattern,
        }
    }

    pub(crate) fn set(&mut self, identifier: Identifier, pattern: Pattern) {
        self.0.insert(identifier, pattern);
    }

    pub(crate) fn extract(&self, pattern: &Pattern) -> crate::Result<Option<Value>> {
        Ok(self.apply(pattern)?.into())
    }

    pub(crate) fn apply(&self, pattern: &Pattern) -> crate::Result<Pattern> {
        match pattern {
            Pattern::Variable(identifier) => {
                let pattern = self.0.get(identifier).ok_or_else(|| {
                    crate::Error::binding(
                        "The pattern contains variables that are not relevant to this binding.",
                    )
                })?;
                self.apply(pattern)
            }
            Pattern::List(patterns, rest) => {
                let mut patterns = patterns
                    .iter()
                    .map(|pattern| self.apply(pattern))
                    .collect::<crate::Result<Vec<_>>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Box<Pattern>>> {
                        match self.apply(&*pattern)? {
                            Pattern::List(mut head, rest) => {
                                patterns.append(&mut head);
                                Ok(rest)
                            }
                            Pattern::Wildcard => Ok(Some(Box::new(Pattern::Wildcard))),
                            _ => panic!("We have unified a list with a non-list value. This should not happen."),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::List(patterns, rest))
            }
            #[cfg(features = "builtin-sets")]
            Pattern::Set(patterns, rest) => {
                let mut patterns = patterns
                    .iter()
                    .map(|pattern| self.apply(pattern))
                    .collect::<crate::Result<Vec<_>>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Box<Pattern>>> {
                        match self.apply(&*pattern)? {
                            Pattern::Set(mut head, rest) => {
                                patterns.append(&mut head);
                                Ok(rest)
                            }
                            Pattern::Wildcard => Ok(Some(Box::new(Pattern::Wildcard))),
                            _ => panic!("We have unified a set with a non-set value. This should not happen."),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::Set(patterns, rest))
            }
            Pattern::Struct(crate::ast::Struct {
                name,
                arity,
                fields,
            }) => {
                let fields = fields
                    .iter()
                    .map(|pattern| self.apply(pattern))
                    .collect::<crate::Result<Vec<_>>>()?;
                Ok(Pattern::Struct(crate::ast::Struct {
                    name: name.clone(),
                    arity: arity.clone(),
                    fields,
                }))
            }
            Pattern::Literal(..) => Ok(pattern.clone()),
            Pattern::Wildcard => Ok(Pattern::Wildcard),
        }
    }
}

impl FromIterator<Identifier> for Binding {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Identifier>,
    {
        Self(
            iter.into_iter()
                .map(|ident| (ident, Pattern::default()))
                .collect(),
        )
    }
}
