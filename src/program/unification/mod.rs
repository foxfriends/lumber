use super::Database;
use crate::ast::*;
use crate::Binding;

type Bindings<'a> = Box<dyn Iterator<Item = Binding> + 'a>;

impl Database<'_> {
    pub(crate) fn unify<'a>(&'a self, question: Body) -> impl Iterator<Item = Binding> + 'a {
        let binding = question.identifiers().collect();
        self.unify_disjunction(question.0, &binding)
    }

    fn unify_disjunction<'a>(
        &'a self,
        disjunction: Disjunction,
        binding: &Binding,
    ) -> Bindings<'a> {
        disjunction
            .cases
            .into_iter()
            .find_map(|case| -> Option<Bindings> {
                let mut bindings = self.unify_conjunction(case, binding).peekable();
                bindings.peek()?;
                Some(Box::new(bindings))
            })
            .unwrap_or(Box::new(std::iter::empty()))
    }

    fn unify_conjunction<'a>(
        &'a self,
        conjunction: Conjunction,
        binding: &Binding,
    ) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding.clone()));
        conjunction
            .terms
            .into_iter()
            .fold(bindings, |bindings, term| {
                Box::new(
                    bindings.flat_map(move |binding| self.unify_procession(term.clone(), &binding)),
                )
            })
    }

    fn unify_procession<'a>(&'a self, procession: Procession, binding: &Binding) -> Bindings<'a> {
        let bindings = Box::new(std::iter::once(binding.clone()));
        procession
            .steps
            .into_iter()
            .fold(bindings, |mut bindings, step| match bindings.next() {
                Some(binding) => self.perform_unification(step, &binding),
                None => Box::new(std::iter::empty()),
            })
    }

    fn perform_unification<'a>(
        &'a self,
        unification: Unification,
        binding: &Binding,
    ) -> Bindings<'a> {
        match unification {
            Unification::Query(query) => todo!(),
            Unification::Body(body) => self.unify_disjunction(body.0, binding),
            Unification::Assumption(output, expression) => {
                Box::new(self.unify_expression(expression, binding).filter_map(
                    move |(binding, pattern)| self.unify_patterns(&output, &pattern, &binding),
                ))
            }
        }
    }

    fn unify_expression<'a>(
        &'a self,
        expression: Expression,
        binding: &Binding,
    ) -> Box<dyn Iterator<Item = (Binding, Pattern)> + 'a> {
        match expression {
            Expression::Operation(pattern, unifications) => Box::new(
                unifications
                    .into_iter()
                    .fold(
                        Box::new(std::iter::once(binding.clone())) as Bindings,
                        |bindings: Bindings, term: Unification| -> Bindings {
                            Box::new(bindings.flat_map(move |binding| {
                                self.perform_unification(term.clone(), &binding)
                            }))
                        },
                    )
                    .map(move |binding| (binding, pattern.clone())),
            ),
            Expression::Value(pattern) => Box::new(std::iter::once((binding.clone(), pattern))),
            Expression::SetAggregation(pattern, body) => {
                let solutions = self
                    .unify_disjunction(body.0, binding)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Box::new(std::iter::once((
                    binding.clone(),
                    Pattern::Set(solutions, None),
                )))
            }
            Expression::ListAggregation(pattern, body) => {
                let solutions = self
                    .unify_disjunction(body.0, binding)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Box::new(std::iter::once((
                    binding.clone(),
                    Pattern::List(solutions, None),
                )))
            }
        }
    }

    fn unify_patterns(&self, lhs: &Pattern, rhs: &Pattern, binding: &Binding) -> Option<Binding> {
        None
    }
}
