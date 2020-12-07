use super::super::{Database, DatabaseDefinition};
use super::{unify_patterns, Bindings};
use crate::ast::*;
use crate::{Binding, Question};
use std::borrow::Cow;

#[cfg(feature = "test-perf")]
struct FlameIterator<I>(I, usize);

#[cfg(feature = "test-perf")]
impl<I> Iterator for FlameIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.1 += 1;
        flame::start("FlameIterator::next");
        let output = self.0.next();
        flame::end("FlameIterator::next");
        flame::dump_html(std::fs::File::create(format!("Flame-{}.html", self.1)).unwrap()).unwrap();
        flame::clear();
        output
    }
}

impl Database<'_> {
    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub(crate) fn unify_question<'a>(
        &'a self,
        question: &'a Question,
    ) -> impl Iterator<Item = Binding> + 'a {
        let body = question.as_ref();
        let answers = self
            .unify_body(body, Cow::Borrowed(&question.initial_binding), true)
            .map(|cow| cow.into_owned()); // TODO: do we even need to owned it here?
        #[cfg(feature = "test-perf")]
        {
            FlameIterator(answers, 0)
        }
        #[cfg(not(feature = "test-perf"))]
        {
            answers
        }
    }

    /// Runs a test. A test does not need to reference public predicates only.
    #[cfg_attr(feature = "test-perf", flamer::flame)]
    pub(crate) fn unify_test<'a>(
        &'a self,
        question: &'a Question,
    ) -> impl Iterator<Item = Binding> + 'a {
        let body = question.as_ref();
        let answers = self
            .unify_body(body, Cow::Borrowed(&question.initial_binding), false)
            .map(|cow| cow.into_owned());
        answers
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_body<'a>(
        &'a self,
        body: &'a Body,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        self.unify_disjunction(&body.0, binding, public)
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_disjunction<'a>(
        &'a self,
        disjunction: &'a Disjunction,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        Box::new(
            disjunction
                .cases
                .iter()
                .map(move |(head, tail)| {
                    let head_bindings = self.unify_conjunction(head, binding.clone(), public);
                    match tail {
                        None => (Box::new(head_bindings), None),
                        Some(tail) => (Box::new(head_bindings), Some(tail)),
                    }
                })
                .scan(false, |skip_rest, (mut head, tail)| {
                    if *skip_rest {
                        return None;
                    }
                    if let Some(binding) = head.next() {
                        if tail.is_some() {
                            *skip_rest = true;
                            Some((
                                Box::new(std::iter::once(binding))
                                    as Box<dyn Iterator<Item = Cow<'a, Binding>>>,
                                tail,
                            ))
                        } else {
                            Some((Box::new(std::iter::once(binding).chain(head)), tail))
                        }
                    } else {
                        Some((Box::new(std::iter::empty()), tail))
                    }
                })
                .fuse()
                .flat_map(move |(head_bindings, tail)| -> Bindings<'a> {
                    match tail {
                        None => Box::new(head_bindings),
                        Some(tail) => Box::new(head_bindings.flat_map(move |binding| {
                            self.unify_conjunction(tail, binding, public)
                        })),
                    }
                }),
        )
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_conjunction<'a>(
        &'a self,
        conjunction: &'a Conjunction,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding));
        conjunction.terms.iter().fold(bindings, |bindings, term| {
            Box::new(bindings.flat_map(move |binding| self.unify_procession(term, binding, public)))
        })
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_procession<'a>(
        &'a self,
        procession: &'a Procession,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding));
        procession
            .steps
            .iter()
            .fold(bindings, |mut bindings, step| match bindings.next() {
                Some(binding) => self.perform_unification(step, binding, public),
                None => Box::new(std::iter::empty()),
            })
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn perform_unification<'a>(
        &'a self,
        unification: &'a Unification,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        match unification {
            Unification::Query(query) => {
                let definition = match self.lookup(query.as_ref(), public) {
                    Some(definition) => definition,
                    None => return Box::new(std::iter::empty()),
                };
                match definition {
                    DatabaseDefinition::Static(definition) => {
                        self.unify_definition(&query, definition, binding)
                    }
                    DatabaseDefinition::Mutable(_definition) => {
                        todo!("Not sure yet how mutable definitions can be handled soundly")
                        // self.unify_definition(&query, &*definition.borrow(), binding)
                    }
                    DatabaseDefinition::Native(native_function) => {
                        let values = query
                            .patterns
                            .iter()
                            .map(|pattern| binding.extract(pattern).unwrap())
                            .collect::<Vec<_>>();
                        Box::new(native_function.call(values).filter_map(move |values| {
                            let values = values
                                .into_iter()
                                .map(Into::into)
                                .zip(query.patterns.iter())
                                .try_fold(binding.clone(), |binding, (lhs, rhs)| {
                                    Some(unify_patterns(
                                        Cow::Borrowed(&lhs),
                                        Cow::Borrowed(rhs),
                                        binding,
                                        &[],
                                    )?)
                                });
                            values
                        }))
                    }
                    _ => unreachable!(),
                }
            }
            Unification::Body(body) => self.unify_body(body, binding, public),
            Unification::Assumption(output, expression) => Box::new(
                self.unify_expression(expression, binding, public)
                    .filter_map(move |(binding, pattern)| {
                        Some(unify_patterns(
                            Cow::Borrowed(&output),
                            Cow::Owned(pattern),
                            binding,
                            &[],
                        )?)
                    }),
            ),
        }
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_definition<'a>(
        &'a self,
        query: &'a Query,
        definition: &'a Definition,
        input_binding: Cow<'a, Binding>,
    ) -> Bindings<'a> {
        Box::new(
            definition
                .iter()
                .map({
                    let input_binding = input_binding.clone();
                    move |(head, kind, body)| {
                        let output_binding = head
                            .identifiers()
                            .chain(body.iter().flat_map(Body::identifiers))
                            .collect::<Binding>();
                        let output_binding = Binding::transfer_from(
                            Cow::Owned(output_binding),
                            &input_binding,
                            &query,
                            &head,
                        );
                        (output_binding, head, *kind, body)
                    }
                })
                .scan(false, |skip_rest, (binding, head, kind, body)| {
                    if *skip_rest {
                        return None;
                    }
                    if binding.is_some() && kind == RuleKind::Once {
                        *skip_rest = true;
                    }
                    Some((binding, head, body))
                })
                .fuse()
                .flat_map(move |(binding, head, body)| {
                    let input_binding = input_binding.clone();
                    Box::new(
                        binding
                            .map(|binding| match body {
                                Some(body) => self.unify_body(body, binding, false),
                                None => Box::new(std::iter::once(binding)),
                            })
                            .into_iter()
                            .flatten()
                            .filter_map(move |output_binding| {
                                Binding::transfer_from(
                                    input_binding.clone(),
                                    &output_binding,
                                    &head,
                                    &query,
                                )
                            }),
                    )
                }),
        )
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_expression<'a>(
        &'a self,
        expression: &'a Expression,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Box<dyn Iterator<Item = (Cow<'a, Binding>, Pattern)> + 'a> {
        match expression {
            Expression::Operation(pattern, unifications) => Box::new(
                unifications
                    .iter()
                    .fold(
                        Box::new(std::iter::once(binding)) as Bindings,
                        |bindings: Bindings, term: &Unification| -> Bindings {
                            Box::new(bindings.flat_map(move |binding| {
                                self.perform_unification(term, binding, public)
                            }))
                        },
                    )
                    .map(move |binding| (binding, pattern.clone())),
            ),
            Expression::Value(pattern) => Box::new(std::iter::once((binding, pattern.clone()))),
            #[cfg(feature = "builtin-sets")]
            Expression::SetAggregation(pattern, body) => {
                let solutions = self
                    .unify_disjunction(&body.0, binding.clone(), public)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Box::new(std::iter::once((binding, Pattern::Set(solutions, None))))
            }
            Expression::ListAggregation(pattern, body) => {
                let solutions = self
                    .unify_body(body, binding.clone(), public)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Box::new(std::iter::once((binding, Pattern::List(solutions, None))))
            }
        }
    }
}
