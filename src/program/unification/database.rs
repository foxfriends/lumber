use super::super::{Database, DatabaseDefinition};
use super::{unify_patterns, Bindings};
use crate::ast::*;
use crate::{Binding, Question};

impl Database<'_> {
    pub(crate) fn unify_question<'a>(
        &'a self,
        question: &'a Question,
    ) -> impl Iterator<Item = Binding> + 'a {
        self.unify(question.as_ref(), true)
    }

    fn unify<'a>(&'a self, question: &'a Body, public: bool) -> impl Iterator<Item = Binding> + 'a {
        let binding = question.identifiers().collect();
        self.unify_disjunction(&question.0, binding, public)
    }

    fn unify_disjunction<'a>(
        &'a self,
        disjunction: &'a Disjunction,
        binding: Binding,
        public: bool,
    ) -> Bindings<'a> {
        disjunction
            .cases
            .iter()
            .find_map(move |case| -> Option<Bindings> {
                let mut bindings = self
                    .unify_conjunction(case, binding.clone(), public)
                    .peekable();
                bindings.peek()?;
                Some(Box::new(bindings))
            })
            .unwrap_or(Box::new(std::iter::empty()))
    }

    fn unify_conjunction<'a>(
        &'a self,
        conjunction: &'a Conjunction,
        binding: Binding,
        public: bool,
    ) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding));
        conjunction.terms.iter().fold(bindings, |bindings, term| {
            Box::new(bindings.flat_map(move |binding| self.unify_procession(term, binding, public)))
        })
    }

    fn unify_procession<'a>(
        &'a self,
        procession: &'a Procession,
        binding: Binding,
        public: bool,
    ) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding.clone()));
        procession
            .steps
            .iter()
            .fold(bindings, |mut bindings, step| match bindings.next() {
                Some(binding) => self.perform_unification(step, binding, public),
                None => Box::new(std::iter::empty()),
            })
    }

    fn perform_unification<'a>(
        &'a self,
        unification: &'a Unification,
        binding: Binding,
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
                    DatabaseDefinition::Native(..) => todo!(),
                    _ => unreachable!(),
                }
            }
            Unification::Body(body) => self.unify_disjunction(&body.0, binding, public),
            Unification::Assumption(output, expression) => Box::new(
                self.unify_expression(expression, binding, public)
                    .filter_map(move |(binding, pattern)| {
                        let occurs = &output.identifiers().collect::<Vec<_>>();
                        Some(unify_patterns(&output, &pattern, binding, occurs)?.1)
                    }),
            ),
        }
    }

    fn unify_definition<'a>(
        &'a self,
        query: &'a Query,
        definition: &'a Definition,
        input_binding: Binding,
    ) -> Bindings<'a> {
        Box::new(definition.iter().flat_map(move |(head, body)| {
            let input_binding = input_binding.clone();
            body.identifiers()
                .collect::<Binding>()
                .transfer_from(&input_binding, &query, &head)
                .map(move |binding| self.unify_disjunction(&body.0, binding, false))
                .into_iter()
                .flatten()
                .filter_map(move |output_binding| {
                    input_binding
                        .clone()
                        .transfer_from(&output_binding, &head, &query)
                })
        }))
    }

    fn unify_expression<'a>(
        &'a self,
        expression: &'a Expression,
        binding: Binding,
        public: bool,
    ) -> Box<dyn Iterator<Item = (Binding, Pattern)> + 'a> {
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
                    .unify_disjunction(&body.0, binding.clone(), public)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Box::new(std::iter::once((binding, Pattern::List(solutions, None))))
            }
        }
    }
}
