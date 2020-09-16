use super::*;
use crate::parser::{Parser, Rule};
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::path::PathBuf;

/// A module within a Lumber program.
#[derive(Clone, Debug)]
pub struct Module {
    /// The path from which to resolve dependencies of this module. If this module was read from
    /// file, this will be a path to the source file. Otherwise, if this module is from a
    /// non-filesystem location, this is simply a directory from which to search for more modules.
    path: PathBuf,
    /// Modules declared in this module.
    submodules: HashMap<Atom, Module>,
    /// Scopes (modules) from which to find implicit imports.
    implicits: HashSet<Scope>,
    /// Predicates which have been imported directly from other modules.
    aliases: HashMap<Handle, Handle>,
    /// Native predicates and functions bound to this module.
    natives: HashSet<Handle>,
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
        let mut implicits = HashSet::new();
        let mut aliases = HashMap::new();
        let mut natives = HashSet::new();
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
                                        match aliases.entry(output) {
                                            Entry::Occupied(entry) => {
                                                context.error_duplicate_import(
                                                    entry.key(),
                                                    entry.get(),
                                                );
                                            }
                                            Entry::Vacant(entry) => {
                                                entry.insert(input);
                                            }
                                        }
                                    }
                                }
                                Err(module) => {
                                    if let Some(module) = implicits.replace(module) {
                                        context.error_duplicate_glob(module);
                                    }
                                }
                            }
                        }
                        Rule::native => {
                            let pair = just!(Rule::handle, pair.into_inner());
                            let handle = Handle::new(pair, context);
                            context.declare_predicate(handle.clone());
                            if let Some(handle) = natives.replace(handle) {
                                context.error_duplicate_native(handle);
                            }
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
                            let output = context.fresh_variable();
                            let head = Query::from_function_head(
                                pairs.next().unwrap(),
                                context,
                                Pattern::Variable(output),
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
                            unifications.push(todo!("Rule::computation"));
                            (head, Body::new_evaluation(unifications))
                        }
                        _ => unreachable!(),
                    };
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
            implicits,
            aliases,
            natives,
            definitions,
        })
    }
}
