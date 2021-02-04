use super::unification::unify_patterns_new_generation;
use crate::program::evaltree::*;
use crate::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// A binding of variables. Not all of the variables are necessarily bound, but together they
/// represent a valid solution to a query.
#[derive(Clone, Debug)]
pub(crate) struct Binding {
    variables: HashMap<Variable, Pattern>,
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
                .variables()
                .map(|var| var.set_current(Some(0)))
                .map(|var| (var.clone(), Pattern::from(PatternKind::Variable(var))))
                .collect(),
            generations: vec![0],
            next_generation: 1,
        }
    }

    pub fn associate_value(&mut self, value: Option<Value>) -> Pattern {
        let pattern: Pattern = value.into();
        let age = Some(self.generation());
        self.variables.extend(
            pattern
                .variables()
                .map(|var| var.set_current(age))
                .map(|var| (var.clone(), Pattern::from(PatternKind::Variable(var)))),
        );
        pattern.default_age(age)
    }

    pub fn generation(&self) -> usize {
        *self.generations.last().unwrap()
    }

    pub fn prev_generation(&self) -> usize {
        self.generations[self.generations.len() - 2]
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub fn start_generation<'b>(
        &self,
        body: Option<&Body>,
        source: &[Pattern],
        destination: &[Pattern],
    ) -> Option<Cow<'b, Self>> {
        let mut binding = self.clone();
        let generation = binding.next_generation;
        binding.generations.push(generation);
        binding.next_generation += 1;
        binding.variables.extend(
            destination
                .iter()
                .flat_map(|pat| pat.variables())
                .chain(body.into_iter().flat_map(|body| body.variables()))
                .map(|var| var.set_current(Some(generation)))
                .map(|var| (var.clone(), Pattern::from(PatternKind::Variable(var)))),
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

    pub fn get(&self, var: &Variable) -> Option<Pattern> {
        let pattern = self.variables.get(var)?;
        match pattern.kind() {
            PatternKind::Variable(new_var) if new_var != var => self.get(new_var),
            _ => Some(pattern.clone()),
        }
    }

    pub fn set(&mut self, var: Variable, pattern: Pattern) {
        self.variables
            .insert(var.set_current(Some(self.generation())), pattern);
    }

    pub fn fresh_variable(&mut self) -> Variable {
        let name = format!("${}", self.variables.len());
        let var = Variable::new(Identifier::new(name), self.generation());
        self.variables.insert(
            var.clone(),
            Pattern::from(PatternKind::Variable(var.clone())),
        );
        var
    }

    pub fn bind(&mut self, name: &str, value: Value) {
        let generation = self.generation();
        let var = self
            .variables
            .keys()
            .find(|var| var.name() == name && var.generation().unwrap_or(generation) == generation)
            .unwrap()
            .clone();
        let pattern = self.associate_value(Some(value));
        *self.variables.get_mut(&var).unwrap() = pattern;
    }

    pub fn extract(&self, pattern: &Pattern) -> crate::Result<Option<Value>> {
        Ok(self.apply(pattern)?.into())
    }

    pub fn apply(&self, pattern: &Pattern) -> crate::Result<Pattern> {
        let age = pattern.age().or_else(|| Some(self.generation()));

        #[cfg(feature = "test-perf")]
        let _guard = {
            let name = match pattern {
                PatternKind::Variable(identifier) => format!("var {}", identifier.name()),
                PatternKind::List(..) => "list".to_owned(),
                PatternKind::Record(..) => "record".to_owned(),
                PatternKind::Struct(s) => format!("struct {}", s.name),
                PatternKind::Literal(..) => "literal".to_owned(),
                PatternKind::All(..) => "all".to_owned(),
                PatternKind::Any(..) => "any".to_owned(),
                _ => format!("{:?}", pattern.to_owned()),
            };
            flame::start_guard(format!("apply({})", name))
        };

        let output = match pattern.kind() {
            PatternKind::Variable(variable) => {
                let variable = variable.set_current(age);
                let pattern = self.variables.get(&variable).ok_or_else(|| {
                    crate::Error::binding(
                        "The pattern contains variables that are not relevant to this binding.",
                    )
                })?;
                return match pattern.kind() {
                    PatternKind::Variable(var) if var == &variable => Ok(pattern.clone()),
                    _ => self.apply(&pattern.default_age(age)),
                };
            }
            PatternKind::List(patterns, rest) => {
                let mut patterns = patterns
                    .iter()
                    .map(|pattern| self.apply(&pattern.default_age(age)))
                    .collect::<crate::Result<Vec<_>>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Pattern>> {
                        let pattern = self.apply(&pattern.default_age(age))?;
                        match pattern.kind() {
                            PatternKind::List(head, rest) => {
                                patterns.append(&mut head.clone());
                                Ok(rest.clone())
                            }
                            PatternKind::Variable(..) => Ok(Some(pattern)),
                            v => panic!("We have unified a list with a non-list value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                PatternKind::List(patterns, rest)
            }
            PatternKind::Record(fields, rest) => {
                let mut fields = fields
                    .iter()
                    .map(|(key, pattern)| Ok((key.clone(), self.apply(&pattern.default_age(age))?)))
                    .collect::<crate::Result<Fields>>()?;
                let rest = rest
                    .as_ref()
                    .map(|pattern| -> crate::Result<Option<Pattern>> {
                        let pattern = self.apply(&pattern.default_age(age))?;
                        match pattern.kind() {
                            PatternKind::Record(head, rest) => {
                                fields.append(&mut head.clone());
                                Ok(rest.clone())
                            }
                            PatternKind::Variable(..) => Ok(Some(pattern)),
                            v => panic!("We have unified a record with a non-record value ({:?}). This should not happen.", v),
                        }
                    })
                    .transpose()?
                    .flatten();
                PatternKind::record(fields, rest)
            }
            PatternKind::Struct(crate::program::evaltree::Struct { name, contents }) => {
                let contents = contents
                    .as_ref()
                    .map(|pattern| self.apply(&pattern.default_age(age)))
                    .transpose()?;
                PatternKind::Struct(crate::program::evaltree::Struct {
                    name: name.clone(),
                    contents,
                })
            }
            PatternKind::Literal(..) => return Ok(pattern.clone()),
            PatternKind::Any(..) => return Ok(pattern.clone()),
            PatternKind::Bound => PatternKind::Bound,
            PatternKind::Unbound => PatternKind::Unbound,
            PatternKind::All(inner) => PatternKind::All(
                inner
                    .iter()
                    .map(|pattern| self.apply(&pattern.default_age(age)))
                    .collect::<crate::Result<Vec<_>>>()?,
            ),
        };
        Ok(Pattern::from(output))
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
                var.generation().unwrap_or_else(|| self.generation()),
                val
            )?;
        }
        write!(f, "}}")
    }
}
