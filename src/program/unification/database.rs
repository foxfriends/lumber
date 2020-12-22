use super::super::{Database, DatabaseDefinition};
use super::{unify_patterns, Bindings};
use crate::ast::*;
use crate::{Binding, Question};
use std::borrow::Cow;

type Evaluation<'a> = (Cow<'a, Pattern>, Bindings<'a>);
type MultipleEvaluations<'a> = (Vec<Cow<'a, Pattern>>, Cow<'a, Binding>);

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
                Some(binding) => self.perform_step(step, binding, public),
                None => Box::new(std::iter::empty()),
            })
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn perform_step<'a>(
        &'a self,
        unification: &'a Step,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        match unification {
            Step::Query(query) => Box::new(
                self.evaluate_expressions(query.args(), binding, public)
                    .flat_map(move |(arguments, binding)| {
                        self.unify_query(query.handle(), arguments, binding, public)
                    }),
            ),
            Step::Relation(None, op, rhs) => {
                match self.resolve_operator(&OpKey::Relation(op.clone(), OpArity::Unary)) {
                    Some(operator) => {
                        let handle = operator.handle();
                        let (pattern, bindings) = self.evaluate_term(rhs, binding, public);
                        Box::new(bindings.flat_map(move |binding| {
                            self.unify_query(handle, vec![pattern.clone()], binding, false)
                        }))
                    }
                    None => Box::new(std::iter::empty()),
                }
            }
            Step::Relation(Some(lhs), op, rhs) => {
                match self.resolve_operator(&OpKey::Relation(op.clone(), OpArity::Binary)) {
                    Some(operator) => {
                        let handle = operator.handle();
                        let (lvar, bindings) = self.evaluate_term(lhs, binding, public);
                        Box::new(bindings.flat_map({
                            let lvar = lvar.clone();
                            move |binding| {
                                let (rvar, bindings) = self.evaluate_term(rhs, binding, public);
                                bindings.flat_map({
                                    let lvar = lvar.clone();
                                    let rvar = rvar.clone();
                                    move |binding| {
                                        self.unify_query(
                                            handle,
                                            vec![lvar.clone(), rvar.clone()],
                                            binding,
                                            false,
                                        )
                                    }
                                })
                            }
                        }))
                    }
                    None => Box::new(std::iter::empty()),
                }
            }
            Step::Body(body) => self.unify_body(body, binding, public),
            Step::Unification(lhs, rhs) => {
                let (lvar, bindings) = self.evaluate_expression(lhs, binding, public);
                Box::new(bindings.flat_map(move |binding| {
                    let (rvar, bindings) = self.evaluate_expression(rhs, binding, public);
                    bindings.flat_map({
                        let lvar = lvar.clone();
                        move |binding| {
                            unify_patterns(Cow::Borrowed(&lvar), Cow::Borrowed(&rvar), binding, &[])
                        }
                    })
                }))
            }
        }
    }

    fn evaluate_expressions<'a>(
        &'a self,
        expressions: &'a [Expression],
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> impl Iterator<Item = MultipleEvaluations<'a>> {
        expressions.iter().fold(
            Box::new(std::iter::once((vec![], binding))) as Box<dyn Iterator<Item = _>>,
            move |bindings: Box<dyn Iterator<Item = MultipleEvaluations<'a>>>, expression| {
                Box::new(bindings.flat_map(move |(mut outputs, binding)| {
                    let (var, bindings) = self.evaluate_expression(expression, binding, public);
                    outputs.push(var);
                    bindings.map(move |binding| (outputs.clone(), binding))
                }))
            },
        )
    }

    fn unify_query<'a>(
        &'a self,
        handle: &Handle,
        args: Vec<Cow<'a, Pattern>>,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Bindings<'a> {
        assert_eq!(handle.arity.len() as usize, args.len());
        let definition = match self.lookup(handle, public) {
            Some(definition) => definition,
            None => return Box::new(std::iter::empty()),
        };
        match definition {
            DatabaseDefinition::Static(definition) => {
                self.unify_definition(definition, args, binding)
            }
            DatabaseDefinition::Mutable(_definition) => {
                todo!("Not sure yet how mutable definitions can be handled soundly")
            }
            DatabaseDefinition::Native(native_function) => {
                let values = args.iter().map(|p| binding.extract(p).unwrap()).collect();
                Box::new(native_function.call(values).filter_map(move |values| {
                    values
                        .into_iter()
                        .map(Into::into)
                        .zip(args.clone().into_iter())
                        .try_fold(binding.clone(), |binding, (lhs, rhs)| {
                            unify_patterns(Cow::Borrowed(&lhs), rhs, binding, &[])
                        })
                }))
            }
            _ => unreachable!(),
        }
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn unify_definition<'a>(
        &'a self,
        definition: &'a Definition,
        expressions: Vec<Cow<'a, Pattern>>,
        input_binding: Cow<'a, Binding>,
    ) -> Bindings<'a> {
        Box::new(
            definition
                .iter()
                .map({
                    let input_binding = input_binding.clone();
                    let expressions = expressions.clone();
                    move |(head, kind, body)| {
                        let output_binding = head
                            .identifiers()
                            .chain(body.iter().flat_map(Body::identifiers))
                            .collect::<Binding>();
                        let output_binding = Binding::transfer_from(
                            Cow::Owned(output_binding),
                            &input_binding,
                            &expressions,
                            &head.patterns.iter().map(Cow::Borrowed).collect::<Vec<_>>(),
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
                    let expressions = expressions.clone();
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
                                    &head.patterns.iter().map(Cow::Borrowed).collect::<Vec<_>>(),
                                    &expressions,
                                )
                            }),
                    )
                }),
        )
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn evaluate_expression<'a>(
        &'a self,
        expression: &'a Expression,
        binding: Cow<'a, Binding>,
        public: bool,
    ) -> Evaluation<'a> {
        let eval = expression
            .climb_operators::<Box<dyn Fn(Cow<'a, Binding>) -> Evaluation<'a>>, _, _, _, _, _>(
                |operator| self.resolve_operator(operator),
                move |term| Box::new(move |binding| self.evaluate_term(term, binding, public)),
                move |term, operator| {
                    Box::new(move |mut binding| {
                        let dest = Pattern::Variable(binding.to_mut().fresh_variable());
                        let (out, bindings) = term(binding);
                        (
                            Cow::Owned(dest.clone()),
                            Box::new(bindings.flat_map(move |binding| {
                                self.unify_query(
                                    operator.handle(),
                                    vec![out.clone(), Cow::Owned(dest.clone())],
                                    binding,
                                    public,
                                )
                            })),
                        )
                    })
                },
                move |lhs, operator, rhs| {
                    let rhs = std::rc::Rc::new(rhs);
                    Box::new(move |mut binding| {
                        let dest = Pattern::Variable(binding.to_mut().fresh_variable());
                        let (lvar, bindings) = lhs(binding);
                        let bindings = Box::new(bindings.flat_map({
                            let rhs = rhs.clone();
                            let dest = dest.clone();
                            move |binding| {
                                let (rvar, bindings) = rhs(binding);
                                bindings.flat_map({
                                    let lvar = lvar.clone();
                                    let rvar = rvar.clone();
                                    let dest = dest.clone();
                                    move |binding| {
                                        self.unify_query(
                                            operator.handle(),
                                            vec![
                                                lvar.clone(),
                                                rvar.clone(),
                                                Cow::Owned(dest.clone()),
                                            ],
                                            binding,
                                            public,
                                        )
                                    }
                                })
                            }
                        }));
                        (Cow::Owned(dest), bindings)
                    })
                },
            );
        match eval {
            Some(eval) => eval(binding),
            // TODO: is wildcard the only thing that can be returned? should we start throwing errors?
            None => (Cow::Owned(Pattern::default()), Box::new(std::iter::empty())),
        }
    }

    #[cfg_attr(feature = "test-perf", flamer::flame)]
    fn evaluate_term<'a>(
        &'a self,
        term: &'a Term,
        mut binding: Cow<'a, Binding>,
        public: bool,
    ) -> Evaluation<'a> {
        match term {
            Term::Expression(expression) => self.evaluate_expression(expression, binding, public),
            Term::PrefixOp(op, rhs) => {
                let dest = Pattern::Variable(binding.to_mut().fresh_variable());
                let (rvar, bindings) = self.evaluate_term(rhs, binding, public);
                let bindings = Box::new(bindings.flat_map({
                    let rvar = rvar.clone();
                    let dest = dest.clone();
                    move |binding| {
                        self.unify_query(
                            op.handle(),
                            vec![rvar.clone(), Cow::Owned(dest.clone())],
                            binding,
                            public,
                        )
                    }
                }));
                (Cow::Owned(dest), bindings)
            }
            Term::InfixOp(lhs, op, rhs) => {
                let dest = Pattern::Variable(binding.to_mut().fresh_variable());
                let (lvar, bindings) = self.evaluate_term(lhs, binding, public);
                let bindings = Box::new(bindings.flat_map({
                    let dest = dest.clone();
                    move |binding| {
                        let (rvar, bindings) = self.evaluate_term(rhs, binding, public);
                        bindings.flat_map({
                            let lvar = lvar.clone();
                            let rvar = rvar.clone();
                            let dest = dest.clone();
                            move |binding| {
                                self.unify_query(
                                    op.handle(),
                                    vec![lvar.clone(), rvar.clone(), Cow::Owned(dest.clone())],
                                    binding,
                                    public,
                                )
                            }
                        })
                    }
                }));
                (Cow::Owned(dest), bindings)
            }
            Term::Value(pattern) => (Cow::Borrowed(pattern), Box::new(std::iter::once(binding))),
            #[cfg(feature = "builtin-sets")]
            Term::SetAggregation(pattern, body) => {
                let solutions = self
                    .unify_body(body, Cow::Borrowed(binding), public)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                Some(Cow::Owned(Pattern::Set(solutions, None)))
            }
            Term::ListAggregation(pattern, body) => {
                let solutions = self
                    .unify_body(body, binding.clone(), public)
                    .map(|binding| binding.apply(&pattern).unwrap())
                    .collect();
                (
                    Cow::Owned(Pattern::List(solutions, None)),
                    Box::new(std::iter::once(binding)),
                )
            }
        }
    }
}
