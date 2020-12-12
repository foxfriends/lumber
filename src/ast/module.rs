use super::*;
use crate::parser::{Parser, Rule};
use std::collections::HashMap;

/// A module within a Lumber program.
#[derive(Clone, Debug)]
pub(crate) struct Module {
    /// Modules declared in this module.
    submodules: HashMap<Atom, Module>,
    /// All predicates defined in this module.
    definitions: HashMap<Handle, Definition>,
    /// Operators defined in this module, corresponding to predicates also defined in this module.
    operators: HashMap<Operator, Handle>,
    /// Unit tests that are defined in this module.
    tests: Vec<Body>,
}

impl Module {
    pub fn new(source_str: &str, context: &mut Context) -> crate::Result<Self> {
        let pairs = Parser::parse_module(source_str)?;
        let pairs = just!(Rule::module, pairs).into_inner();

        let mut submodules = HashMap::<Atom, Module>::new();
        let mut definitions = HashMap::<Handle, Definition>::new();
        let mut operators = HashMap::<Operator, Handle>::new();
        let mut tests: Vec<Body> = vec![];

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
                                    for alias in unpacked {
                                        match alias {
                                            Alias::Predicate { input, output } => {
                                                context.declare_alias(output.clone(), input.clone())
                                            }
                                            Alias::Operator { name, scope } => {
                                                context.declare_operator_alias(name, scope)
                                            }
                                        }
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
                        Rule::op => {
                            if let Some((operator, handle)) = Operator::new(pair, context) {
                                context.declare_operator(operator.clone(), handle.clone());
                                operators.insert(operator, handle);
                            }
                        }
                        Rule::test => {
                            let pair = just!(Rule::body, pair.into_inner());
                            match Body::new(pair, context) {
                                Some(body) => tests.push(body),
                                None => continue,
                            }
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
                            let query = Head::new(pair, context);
                            // TODO: is there a way to define "Once" facts?
                            (query, RuleKind::Multi, None)
                        }
                        Rule::rule => {
                            let mut pairs = pair.into_inner();
                            let head = Head::new(pairs.next().unwrap(), context);
                            let kind = match pairs.next().unwrap().as_rule() {
                                Rule::rule_multi => RuleKind::Multi,
                                Rule::rule_once => RuleKind::Once,
                                _ => unreachable!(),
                            };
                            match Body::new(pairs.next().unwrap(), context) {
                                Some(body) => (head, kind, Some(body)),
                                None => continue,
                            }
                        }
                        _ => unreachable!(),
                    };
                    if let Some(body) = &body {
                        body.check_variables(&head, context);
                    } else {
                        head.check_variables(context);
                    }
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
            operators,
            tests,
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
        for test in self.tests.iter_mut() {
            for handle in test.handles_mut() {
                if let Some(resolved) = context.resolve_handle(handle) {
                    *handle = resolved.clone();
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

    pub fn take_tests(&mut self) -> Vec<Body> {
        self.submodules
            .iter_mut()
            .flat_map(|(_, module)| module.take_tests())
            .chain(self.tests.drain(..))
            .collect()
    }
}
