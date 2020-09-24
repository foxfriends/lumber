use super::*;
use crate::parser::{Parser, Rule};
use std::collections::HashMap;
use std::path::PathBuf;

/// A module within a Lumber program.
#[derive(Clone, Debug)]
pub struct Module {
    /// The path from which to resolve dependencies of this module. If this module was read from
    /// file, this will be a path to the resolved file. Otherwise, if this module is from a
    /// non-filesystem location, this is simply a directory from which to search for more modules.
    path: PathBuf,
    /// Modules declared in this module.
    submodules: HashMap<Atom, Module>,
    /// All predicates and functions defined in this module.
    definitions: HashMap<Handle, Definition>,
}

impl Module {
    pub(crate) fn new(
        path: PathBuf,
        source_str: &str,
        context: &mut Context,
    ) -> crate::Result<Self> {
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
                            let atom = context.atomizer.atomize(atom);
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
                    let (head, body) = match pair.as_rule() {
                        Rule::fact => {
                            let pair = just!(pair.into_inner());
                            let query = Query::from_head(pair, context);
                            (query, Body::default())
                        }
                        Rule::rule => {
                            let mut pairs = pair.into_inner();
                            let head = Query::from_head(pairs.next().unwrap(), context);
                            if let Some(body) = Body::new(pairs.next().unwrap(), context) {
                                (head, body)
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
                            match Computation::new(pairs.next().unwrap(), context) {
                                Some(computation) => {
                                    unifications.push(Unification::Assumption(output, computation))
                                }
                                None => continue,
                            }
                            (head, Body::new_evaluation(unifications))
                        }
                        _ => unreachable!(),
                    };
                    context.declare_predicate(head.as_ref().clone());
                    definitions
                        .entry(head.as_ref().clone())
                        .or_default()
                        .insert(head, body);
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }

        Ok(Self {
            path,
            submodules,
            definitions,
        })
    }

    pub(crate) fn resolve_scopes(&mut self, context: &mut Context) {
        for (name, module) in self.submodules.iter_mut() {
            context.resolve_scopes(module, name.clone());
        }
        for definition in self.definitions.values_mut() {
            for body in definition.bodies_mut() {
                for handle in body.handles_mut() {
                    if let Some(resolved) = context.resolve_handle(handle) {
                        *handle = resolved;
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

    pub(crate) fn into_definitions(self) -> Box<dyn Iterator<Item = (Handle, Definition)>> {
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
