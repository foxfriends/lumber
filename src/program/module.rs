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
    pub(crate) fn new<'i>(
        path: PathBuf,
        source_str: &'i str,
        context: &mut Context<'i>,
    ) -> crate::Result<Self> {
        let pairs = Parser::parse_module(source_str)?;
        let pairs = just!(Rule::module, pairs).into_inner();

        let mut implicits = HashSet::new();
        let mut aliases = HashMap::new();
        let mut natives = HashSet::new();
        let mut clauses = vec![];

        for pair in pairs {
            match pair.as_rule() {
                Rule::directive => {
                    let pair = just!(Rule::instruction, pair.into_inner());
                    let pair = just!(pair.into_inner());
                    match pair.as_rule() {
                        Rule::mod_ => {
                            let atom = just!(Rule::atom, pair.into_inner());
                            let atom = context.atomizer.atomize(atom);
                            context.declare_module(atom);
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
                            if let Some(handle) = natives.replace(Handle::new(pair, context)) {
                                context.error_duplicate_native(handle);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Rule::clause => clauses.push(pair),
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }

        todo!()
    }
}
