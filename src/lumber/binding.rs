use super::Value;
use crate::ast::*;
use crate::program::unification::unify_patterns;
use std::collections::HashMap;
use std::iter::FromIterator;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Default, Clone, Debug)]
pub struct Binding(pub(crate) HashMap<Identifier, Pattern>);

impl Binding {
    pub(crate) fn transfer_from(
        self,
        input_binding: &Self,
        source: &Query,
        destination: &Query,
    ) -> Option<Self> {
        source
            .patterns
            .iter()
            .zip(destination.patterns.iter())
            .try_fold(self, |binding, (source, destination)| {
                let applied = input_binding.apply(source).unwrap();
                let (_, binding) = unify_patterns(&applied, destination, binding, &[])?;
                Some(binding)
            })
    }

    pub(crate) fn get(&self, identifier: &Identifier) -> Option<&Pattern> {
        let pattern = self.0.get(identifier)?;
        match &pattern {
            Pattern::Variable(identifier) => self.get(identifier),
            _ => Some(pattern),
        }
    }

    pub(crate) fn set(&mut self, identifier: Identifier, pattern: Pattern) {
        self.0.insert(identifier, pattern);
    }

    pub(crate) fn fresh_variable(&mut self) -> Identifier {
        let var = Identifier::new(format!("##{}", self.0.len()));
        self.0.insert(var.clone(), Pattern::Wildcard);
        var
    }

    pub(crate) fn bind(&mut self, variable: &str, value: Value) {
        let identifier = self
            .0
            .keys()
            .find(|id| id.name() == variable)
            .unwrap()
            .clone();
        self.set(identifier, Some(value).into());
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
            Pattern::Record(fields, rest) => {
                let mut fields = fields
                    .iter()
                    .map(|(key, patterns)| {
                        Ok((
                            key.clone(),
                            patterns
                                .iter()
                                .map(|pattern| self.apply(pattern))
                                .collect::<crate::Result<_>>()?,
                        ))
                    })
                    .collect::<crate::Result<Fields>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Box<Pattern>>> {
                        match self.apply(&*pattern)? {
                            Pattern::Record(mut head, rest) => {
                                fields.append(&mut head);
                                Ok(rest)
                            }
                            Pattern::Wildcard => Ok(Some(Box::new(Pattern::Wildcard))),
                            _ => panic!("We have unified a record with a non-record value. This should not happen."),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::Record(fields, rest))
            }
            Pattern::Struct(crate::ast::Struct {
                name,
                patterns,
                fields,
            }) => {
                let patterns = patterns
                    .iter()
                    .map(|pattern| self.apply(pattern))
                    .collect::<crate::Result<Vec<_>>>()?;
                let fields = fields
                    .iter()
                    .map(|(field, patterns)| {
                        Ok((
                            field.clone(),
                            patterns
                                .iter()
                                .map(|pattern| self.apply(pattern))
                                .collect::<crate::Result<Vec<_>>>()?,
                        ))
                    })
                    .collect::<crate::Result<Fields>>()?;
                Ok(Pattern::Struct(crate::ast::Struct {
                    name: name.clone(),
                    patterns,
                    fields,
                }))
            }
            Pattern::Literal(..) => Ok(pattern.clone()),
            Pattern::Any(..) => Ok(pattern.clone()),
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
