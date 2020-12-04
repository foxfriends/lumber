use super::Value;
use crate::ast::*;
use crate::program::unification::unify_patterns;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Default, Clone, Debug)]
pub struct Binding(pub(crate) HashMap<Identifier, Rc<Pattern>>);

impl Binding {
    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub(crate) fn transfer_from<'a, 'b>(
        mut output_binding: Cow<'b, Self>,
        input_binding: &Self,
        source: &'a Query,
        destination: &'a Query,
    ) -> Option<Cow<'b, Self>> {
        let mut source_patterns: Vec<_> = source
            .patterns
            .iter()
            .map(|source| input_binding.apply(source).unwrap())
            .collect();
        let identifiers_map = source_patterns
            .iter()
            .flat_map(Pattern::identifiers)
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|ident| (ident, output_binding.to_mut().fresh_variable()))
            .collect::<HashMap<_, _>>();
        for pattern in &mut source_patterns {
            for identifier in pattern.identifiers_mut() {
                *identifier = identifiers_map.get(identifier).unwrap().clone();
            }
        }
        source_patterns
            .into_iter()
            .zip(destination.patterns.iter())
            .try_fold(output_binding, |binding, (source, destination)| {
                unify_patterns(Cow::Owned(source), Cow::Borrowed(destination), binding, &[])
            })
    }

    pub(crate) fn get(&self, identifier: &Identifier) -> Option<Rc<Pattern>> {
        let pattern = self.0.get(identifier)?;
        match pattern.as_ref() {
            Pattern::Variable(identifier) => self.get(identifier),
            _ => Some(pattern.clone()),
        }
    }

    pub(crate) fn set(&mut self, identifier: Identifier, pattern: Pattern) -> Rc<Pattern> {
        let rc = Rc::new(pattern);
        self.0.insert(identifier, rc.clone());
        rc
    }

    pub(crate) fn fresh_variable(&mut self) -> Identifier {
        let name = format!("##{}", self.0.len());
        let var = Identifier::new(name.clone());
        self.0.insert(
            var.clone(),
            Rc::new(Pattern::Wildcard(Identifier::wildcard(name))),
        );
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
        #[cfg(feature = "test-perf")]
        let _guard = {
            let name = match pattern {
                Pattern::Variable(identifier) => format!("var {}", identifier.name()),
                Pattern::Wildcard(identifier) => format!("wild {}", identifier.name()),
                Pattern::List(..) => "list".to_owned(),
                #[cfg(feature = "builtin-sets")]
                Pattern::Set(..) => "set".to_owned(),
                Pattern::Record(..) => "record".to_owned(),
                Pattern::Struct(s) => format!("struct {}", s.name),
                Pattern::Literal(..) => "literal".to_owned(),
                Pattern::All(..) => "all".to_owned(),
                Pattern::Any(..) => "any".to_owned(),
                _ => format!("{:?}", pattern.to_owned()),
            };
            flame::start_guard(format!("apply({})", name))
        };

        match pattern {
            Pattern::Variable(identifier) => {
                let pattern = self.0.get(identifier).ok_or_else(|| {
                    crate::Error::binding(
                        "The pattern contains variables that are not relevant to this binding.",
                    )
                })?;
                match self.apply(pattern) {
                    Ok(Pattern::Wildcard(..)) => Ok(Pattern::Variable(identifier.clone())),
                    pattern => pattern,
                }
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
                            pat @ Pattern::Variable(..) | pat @ Pattern::Wildcard(..) => Ok(Some(Box::new(pat))),
                            v => panic!("We have unified a list with a non-list value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::List(patterns, rest))
            }
            #[cfg(feature = "builtin-sets")]
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
                            pat @ Pattern::Variable(..) | pat @ Pattern::Wildcard(..) => Ok(Some(Box::new(pat))),
                            v => panic!("We have unified a set with a non-set value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::Set(patterns, rest))
            }
            Pattern::Record(fields, rest) => {
                let mut fields = fields
                    .iter()
                    .map(|(key, pattern)| Ok((key.clone(), self.apply(pattern)?)))
                    .collect::<crate::Result<Fields>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Box<Pattern>>> {
                        match self.apply(&*pattern)? {
                            Pattern::Record(mut head, rest) => {
                                fields.append(&mut head);
                                Ok(rest)
                            }
                            pat @ Pattern::Variable(..) | pat @ Pattern::Wildcard(..) => Ok(Some(Box::new(pat))),
                            v => panic!("We have unified a record with a non-record value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::Record(fields, rest))
            }
            Pattern::Struct(crate::ast::Struct { name, contents }) => {
                let contents = contents
                    .as_deref()
                    .map(|contents| self.apply(&contents))
                    .transpose()?
                    .map(Box::new);
                Ok(Pattern::Struct(crate::ast::Struct {
                    name: name.clone(),
                    contents,
                }))
            }
            Pattern::Literal(..) => Ok(pattern.clone()),
            Pattern::Any(..) => Ok(pattern.clone()),
            Pattern::Bound => Ok(Pattern::Bound),
            Pattern::Unbound => Ok(Pattern::Unbound),
            Pattern::Wildcard(id) => Ok(Pattern::Wildcard(id.clone())),
            Pattern::All(inner) => Ok(Pattern::All(
                inner
                    .iter()
                    .map(|pattern| self.apply(&pattern))
                    .collect::<crate::Result<Vec<_>>>()?,
            )),
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
                .map(|ident| (ident, Rc::new(Pattern::default())))
                .collect(),
        )
    }
}
