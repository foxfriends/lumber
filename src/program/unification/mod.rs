use super::{Database, DatabaseDefinition};
use crate::ast::*;
use crate::{Binding, Question};

type Bindings<'a> = Box<dyn Iterator<Item = Binding> + 'a>;

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

// TODO: unification of sets will end up being a big change, as set unification is not deterministic.
// TODO: This function could be wrapped so it does not return the output pattern, as that is only really
//       used internally.
pub(crate) fn unify_patterns(
    lhs: &Pattern,
    rhs: &Pattern,
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Pattern, Binding)> {
    match (lhs, rhs) {
        // Unifying wildcards provides no additional info, and always succeeds.
        (Pattern::Wildcard, other) | (other, Pattern::Wildcard) => Some((other.clone(), binding)),
        // Unifying a variable with itself succeeds with no additional info.
        (Pattern::Variable(lhs), Pattern::Variable(rhs)) if lhs == rhs => {
            // We don't need to use occurs check here because `A <- A` is allowed, despite
            // `A` being in the occurs list already.
            Some((Pattern::Variable(*lhs), binding))
        }
        // Unifying a variable with a different variable, we use the natural order of variables
        // to designate one as the source of truth and the other as a reference.
        (Pattern::Variable(lhs), Pattern::Variable(rhs)) => {
            if occurs.contains(lhs) || occurs.contains(rhs) {
                return None;
            }
            let lhs_pat = binding.get(*lhs).clone();
            let rhs_pat = binding.get(*rhs).clone();
            let mut occurs = occurs.to_owned();
            occurs.push(*lhs);
            occurs.push(*rhs);
            let (pattern, mut binding) = unify_patterns(&lhs_pat, &rhs_pat, binding, &occurs)?;
            let min = Identifier::min(*lhs, *rhs);
            let max = Identifier::max(*lhs, *rhs);
            binding.set(min, pattern.clone());
            binding.set(max, Pattern::Variable(min));
            Some((pattern, binding))
        }
        // A variable unified with a value should attempt to dereference the variable and then
        // unify. If that succeeds, the variable is replaced with the binding.
        (Pattern::Variable(var), pattern) | (pattern, Pattern::Variable(var)) => {
            if occurs.contains(var) {
                return None;
            }
            let var_pat = binding.get(*var).clone();
            let mut occurs = occurs.to_owned();
            occurs.push(*var);
            let (pattern, mut binding) = unify_patterns(&var_pat, pattern, binding, &occurs)?;
            binding.set(*var, pattern.clone());
            Some((pattern, binding))
        }
        // Literals must match exactly.
        (Pattern::Literal(lhs), Pattern::Literal(rhs)) if lhs == rhs => {
            Some((Pattern::Literal(lhs.clone()), binding.clone()))
        }
        (Pattern::Literal(..), Pattern::Literal(..)) => None,
        // Structs must match in name and arity, and then all their fields must match as well.
        (Pattern::Struct(lhs), Pattern::Struct(rhs))
            if lhs.name == rhs.name && lhs.arity == rhs.arity =>
        {
            let (fields, binding) = unify_sequence(&lhs.fields, &rhs.fields, binding, occurs)?;
            Some((
                Pattern::Struct(Struct {
                    name: lhs.name.clone(),
                    arity: lhs.arity.clone(),
                    fields,
                }),
                binding,
            ))
        }
        (Pattern::Struct(..), Pattern::Struct(..)) => None,
        // If neither list has a tail, the heads must match, similar to struct fields.
        (Pattern::List(lhs, None), Pattern::List(rhs, None)) => {
            let (fields, binding) = unify_sequence(&lhs, &rhs, binding, occurs)?;
            Some((Pattern::List(fields, None), binding))
        }
        // If only one list has a tail, the tail unifies with whatever the head does
        // not already cover.
        (other @ Pattern::List(full, None), Pattern::List(head, Some(tail)))
        | (Pattern::List(head, Some(tail)), other @ Pattern::List(full, None)) => {
            match tail.as_ref() {
                Pattern::Variable(ident) => {
                    let (output, tail, binding) = unify_prefix(head, full, binding, occurs)?;
                    let tail_pat = binding.get(*ident).clone();
                    let mut occurs = occurs.to_owned();
                    occurs.push(*ident);
                    let (tail, binding) =
                        unify_patterns(&Pattern::List(tail, None), &tail_pat, binding, &occurs)?;
                    Some((Pattern::List(output, Some(Box::new(tail))), binding))
                }
                Pattern::Wildcard => {
                    let (mut output, mut tail, binding) =
                        unify_prefix(head, full, binding, occurs)?;
                    output.append(&mut tail);
                    Some((Pattern::List(output, None), binding))
                }
                Pattern::List(cont, tail) => {
                    let mut combined = head.to_owned();
                    combined.extend_from_slice(&cont);
                    let lhs = Pattern::List(combined, tail.clone());
                    unify_patterns(&lhs, other, binding, occurs)
                }
                // If the tail cannot unify with a list, then there is a problem.
                _ => None,
            }
        }
        // If both lists have tails, unify the prefixes of the heads, then we'll have
        // one list and one pattern, which can be unified.
        (Pattern::List(lhs, Some(lhs_tail)), Pattern::List(rhs, Some(rhs_tail))) => {
            let (unified, remaining, binding) = unify_prefix(lhs, rhs, binding, occurs)?;
            // The shorter one is the one that is now "done", so we match it's tail with
            // the remaining head and tail of the other list.
            let (suffix, binding) = if lhs.len() < rhs.len() {
                unify_patterns(
                    lhs_tail.as_ref(),
                    &Pattern::List(remaining, Some(rhs_tail.clone())),
                    binding,
                    occurs,
                )?
            } else {
                unify_patterns(
                    &Pattern::List(remaining, Some(lhs_tail.clone())),
                    rhs_tail.as_ref(),
                    binding,
                    occurs,
                )?
            };
            Some((Pattern::List(unified, Some(Box::new(suffix))), binding))
        }
        // Otherwise, it's a failure!
        _ => None,
    }
}

fn unify_sequence(
    lhs: &[Pattern],
    rhs: &[Pattern],
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Vec<Pattern>, Binding)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    lhs.iter()
        .zip(rhs.iter())
        .try_fold((vec![], binding), |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(lhs, rhs, binding, occurs)?;
            patterns.push(pattern);
            Some((patterns, binding))
        })
}

fn unify_prefix(
    lhs: &[Pattern],
    rhs: &[Pattern],
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Vec<Pattern>, Vec<Pattern>, Binding)> {
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(lhs, rhs, binding, occurs)?;
            patterns.push(pattern);
            Some((patterns, binding))
        },
    )?;
    if lhs.len() < rhs.len() {
        Some((head, rhs[lhs.len()..].to_owned(), binding))
    } else {
        Some((head, lhs[rhs.len()..].to_owned(), binding))
    }
}
