use super::*;
use crate::parser::{Parser, Rule};
use std::collections::HashMap;

/// A module within a Lumber program.
#[derive(Clone, Debug)]
pub(crate) struct Module {
    /// Modules declared in this module.
    submodules: HashMap<Atom, Module>,
    /// All predicates and functions defined in this module.
    definitions: HashMap<Handle, Definition>,
}

impl Module {
    pub fn new(source_str: &str, context: &mut Context) -> crate::Result<Self> {
        let pairs = Parser::parse_module(source_str)?;
        let pairs = just!(Rule::module, pairs).into_inner();

        let mut submodules = HashMap::new();
        let mut definitions = HashMap::<Handle, Definition>::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::directive => {
                    let pair = just!(Rule::instruction, pair.into_inner());
                    let pair = just!(pair.into_inner());
                    match pair.as_rule() {
                        Rule::mod_ => {
                            let atom = just!(Rule::atom, pair.into_inner());
                            let atom = Atom::new(atom);
                            if let Some(module) = context.add_module(atom.clone())? {
                                submodules.insert(atom, module);
                            }
                        }
                        Rule::pub_ => {
                            let handle = just!(Rule::handle, pair.into_inner());
                            let handle = Handle::new(handle, context);
                            context.declare_export(handle);
                        }
                        Rule::use_ => {
                            let handle = just!(Rule::multi_handle, pair.into_inner());
                            match Alias::unpack_multiple(handle, context) {
                                Ok(unpacked) => {
                                    for Alias { input, output } in unpacked {
                                        context.declare_alias(output.clone(), input.clone());
                                    }
                                }
                                Err(module) => context.import_glob(module),
                            }
                        }
                        Rule::mut_ => {
                            let handle = just!(Rule::handle, pair.into_inner());
                            let handle = Handle::new(handle, context);
                            context.declare_mutable(handle);
                        }
                        Rule::inc => {
                            let handle = just!(Rule::handle, pair.into_inner());
                            let handle = Handle::new(handle, context);
                            context.declare_incomplete(handle);
                        }
                        Rule::nat => {
                            let pair = just!(Rule::handle, pair.into_inner());
                            let handle = Handle::new(pair, context);
                            context.declare_native(handle.clone());
                        }
                        _ => unreachable!(),
                    }
                }
                Rule::clause => {
                    context.reset_environment();
                    let pair = just!(pair.into_inner());
                    let (head, kind, body) = match pair.as_rule() {
                        Rule::fact => {
                            let pair = just!(pair.into_inner());
                            let query = Query::from_head(pair, context);
                            // TODO: is there a way to define "Once" rules?
                            (query, RuleKind::Multi, Body::default())
                        }
                        Rule::rule => {
                            let mut pairs = pair.into_inner();
                            let head = Query::from_head(pairs.next().unwrap(), context);
                            let kind = match pairs.next().unwrap().as_rule() {
                                Rule::rule_multi => RuleKind::Multi,
                                Rule::rule_once => RuleKind::Once,
                                _ => unreachable!(),
                            };
                            if let Some(body) = Body::new(pairs.next().unwrap(), context) {
                                (head, kind, body)
                            } else {
                                continue;
                            }
                        }
                        Rule::function => {
                            let mut pairs = pair.into_inner();
                            let output = Pattern::Variable(context.fresh_variable());
                            let head = Query::from_function_head(
                                pairs.next().unwrap(),
                                context,
                                output.clone(),
                            );
                            let kind = match pairs.next().unwrap().as_rule() {
                                Rule::function_multi => RuleKind::Multi,
                                Rule::function_once => RuleKind::Once,
                                _ => unreachable!(),
                            };
                            let mut pairs = just!(Rule::evaluation, pairs).into_inner();
                            let mut unifications = vec![];
                            while pairs.peek().unwrap().as_rule() == Rule::assumption {
                                let unification = match Unification::from_assumption(
                                    pairs.next().unwrap(),
                                    context,
                                ) {
                                    None => continue,
                                    Some(unification) => unification,
                                };
                                unifications.push(unification);
                            }
                            match Expression::new_operation(pairs.next().unwrap(), context) {
                                Some(expression) => {
                                    unifications.push(Unification::Assumption(output, expression))
                                }
                                None => continue,
                            }
                            (head, kind, Body::new_evaluation(unifications))
                        }
                        _ => unreachable!(),
                    };
                    body.check_variables(&head, context);
                    context.declare_predicate(head.as_ref().clone());
                    definitions
                        .entry(head.as_ref().clone())
                        .or_default()
                        .insert(head, kind, body);
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }

        Ok(Self {
            submodules,
            definitions,
        })
    }

    pub fn resolve_scopes(&mut self, context: &mut Context) {
        for (name, module) in self.submodules.iter_mut() {
            context.resolve_scopes(module, name.clone());
        }
        for definition in self.definitions.values_mut() {
            for body in definition.bodies_mut() {
                for handle in body.handles_mut() {
                    if let Some(resolved) = context.resolve_handle(handle) {
                        *handle = resolved.clone();
                    }
                }
            }
        }
        let definitions = std::mem::take(&mut self.definitions);
        self.definitions = definitions
            .into_iter()
            .map(|(handle, definition)| {
                (
                    context.resolve_handle(&handle).unwrap_or(handle),
                    definition,
                )
            })
            .collect();
    }

    pub fn into_definitions(self) -> Box<dyn Iterator<Item = (Handle, Definition)>> {
        Box::new(
            self.definitions.into_iter().chain(
                self.submodules
                    .into_iter()
                    .map(|(_, value)| value)
                    .flat_map(Self::into_definitions),
            ),
        )
    }
}
