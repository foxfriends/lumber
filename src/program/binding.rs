use super::unification::unify_patterns_new_generation;
use crate::program::evaltree::*;
use crate::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Clone, Debug)]
pub(crate) struct Binding {
    variables: HashMap<Variable, Rc<Pattern>>,
    generations: Vec<usize>,
    next_generation: usize,
}

#[cfg(test)]
impl Default for Binding {
    fn default() -> Self {
        Self {
            variables: HashMap::default(),
            generations: vec![0],
            next_generation: 1,
        }
    }
}

impl Binding {
    pub fn new(body: &Body) -> Self {
        Self {
            variables: body
                .variables(0)
                .into_iter()
                .map(|var| (var.clone(), Rc::new(Pattern::Variable(var))))
                .collect(),
            generations: vec![0],
            next_generation: 1,
        }
    }

    pub fn generation(&self) -> usize {
        *self.generations.last().unwrap()
    }

    pub fn prev_generation(&self) -> usize {
        self.generations[self.generations.len() - 2]
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub fn start_generation<'a, 'b>(
        &self,
        body: Option<&Body>,
        source: &[Cow<'a, Pattern>],
        destination: &[Cow<'a, Pattern>],
    ) -> Option<Cow<'b, Self>> {
        let mut binding = self.clone();
        let generation = binding.next_generation;
        binding.generations.push(generation);
        binding.next_generation += 1;
        binding.variables.extend(
            destination
                .iter()
                .flat_map(|pat| pat.variables(generation))
                .chain(body.into_iter().flat_map(|body| body.variables(generation)))
                .map(|var| (var.clone(), Rc::new(Pattern::Variable(var)))),
        );
        source.iter().zip(destination.iter()).try_fold(
            Cow::Owned(binding),
            |binding, (source, destination)| {
                unify_patterns_new_generation(source.clone(), destination.clone(), binding)
            },
        )
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub fn end_generation(mut self) -> Self {
        self.generations.pop();
        self
    }

    pub fn get(&self, var: &Variable) -> Option<Rc<Pattern>> {
        let pattern = self.variables.get(var)?;
        match pattern.as_ref() {
            Pattern::Variable(new_var) if new_var != var => self.get(new_var),
            _ => Some(pattern.clone()),
        }
    }

    pub fn set(&mut self, var: Variable, pattern: Pattern) -> Rc<Pattern> {
        let rc = Rc::new(pattern);
        self.variables
            .insert(var.set_current(self.generation()), rc.clone());
        rc
    }

    pub fn fresh_variable(&mut self) -> Variable {
        let name = format!("${}", self.variables.len());
        let var = Variable::new(Identifier::new(name), self.generation());
        self.variables
            .insert(var.clone(), Rc::new(Pattern::Variable(var.clone())));
        var
    }

    pub fn bind(&mut self, name: &str, value: Value) {
        let var = self
            .variables
            .keys()
            .find(|var| {
                var.name() == name && var.generation(self.generation()) == self.generation()
            })
            .unwrap()
            .clone();
        *self.variables.get_mut(&var).unwrap() = Rc::new(Some(value).into());
    }

    pub fn extract(&self, pattern: &Pattern) -> crate::Result<Option<Value>> {
        Ok(self.apply(pattern)?.into())
    }

    pub fn apply(&self, pattern: &Pattern) -> crate::Result<Pattern> {
        #[cfg(feature = "test-perf")]
        let _guard = {
            let name = match pattern {
                Pattern::Variable(identifier) => format!("var {}", identifier.name()),
                Pattern::List(..) => "list".to_owned(),
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
            Pattern::Variable(variable) => {
                let variable = variable.set_current(self.generation());
                let pattern = self.variables.get(&variable).ok_or_else(|| {
                    crate::Error::binding(
                        "The pattern contains variables that are not relevant to this binding.",
                    )
                })?;
                match pattern.as_ref() {
                    Pattern::Variable(var) if var == &variable => {
                        Ok(Pattern::Variable(var.clone()))
                    }
                    _ => self.apply(pattern),
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
                            pat @ Pattern::Variable(..) => Ok(Some(Box::new(pat))),
                            v => panic!("We have unified a list with a non-list value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::List(patterns, rest))
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
                            pat @ Pattern::Variable(..) => Ok(Some(Box::new(pat))),
                            v => panic!("We have unified a record with a non-record value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                Ok(Pattern::Record(fields, rest))
            }
            Pattern::Struct(crate::program::evaltree::Struct { name, contents }) => {
                let contents = contents
                    .as_deref()
                    .map(|contents| self.apply(&contents))
                    .transpose()?
                    .map(Box::new);
                Ok(Pattern::Struct(crate::program::evaltree::Struct {
                    name: name.clone(),
                    contents,
                }))
            }
            Pattern::Literal(..) => Ok(pattern.clone()),
            Pattern::Any(..) => Ok(pattern.clone()),
            Pattern::Bound => Ok(Pattern::Bound),
            Pattern::Unbound => Ok(Pattern::Unbound),
            Pattern::All(inner) => Ok(Pattern::All(
                inner
                    .iter()
                    .map(|pattern| self.apply(&pattern))
                    .collect::<crate::Result<Vec<_>>>()?,
            )),
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.variables.is_empty() {
            return write!(f, "Binding {{}}");
        }
        writeln!(f, "Binding {{")?;
        for (var, val) in &self.variables {
            writeln!(
                f,
                "\t{} ({}) = {}",
                var,
                var.generation(self.generation()),
                val
            )?;
        }
        write!(f, "}}")
    }
}
