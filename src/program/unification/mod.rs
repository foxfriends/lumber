use super::Database;
use crate::ast::*;
use crate::Binding;

type Bindings<'a> = Box<dyn Iterator<Item = Binding> + 'a>;

impl Database<'_> {
    pub(crate) fn unify<'a>(&'a self, question: Body) -> impl Iterator<Item = Binding> + 'a {
        let binding = question
            .identifiers()
            .map(|identifier| self.variables[Into::<usize>::into(identifier)].clone())
            .collect();
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
        todo!()
    }
}
