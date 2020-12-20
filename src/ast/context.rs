use super::*;
use crate::program::*;
use crate::{Lumber, Question};
use pest::Span;
use ramp::Int;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct Context<'p> {
    pub(crate) libraries: HashMap<Atom, Database<'p>>,
    pub(crate) root_path: PathBuf,
    pub(crate) current_scope: Scope,
    pub(crate) current_environment: HashMap<String, Identifier>,
    pub(crate) modules: HashMap<Scope, ModuleHeader>,
    pub(crate) errors: HashMap<Scope, Vec<crate::Error>>,
}

impl<'p> Context<'p> {
    pub(crate) fn with_core() -> Self {
        let mut context = Self::default();
        crate::core::LIB.with(|lib| {
            let core = Atom::from("core");
            let (modules, library) = lib.clone().into_library("core");
            context.libraries.insert(core, library);
            context.modules = modules;
        });
        context
    }

    pub(crate) fn compile(
        mut self,
        root_path: PathBuf,
        source: &str,
        natives: HashMap<Handle, NativeFunction<'p>>,
        run_tests: bool,
    ) -> crate::Result<Lumber<'p>> {
        self.root_path = root_path;
        if self.root_path.exists() && std::fs::metadata(&self.root_path)?.is_file() {
            self.root_path.pop();
        }
        self.modules
            .insert(Scope::default(), ModuleHeader::new(Scope::default()));

        let mut root_module = Module::new(source, &mut self)?;
        let native_handles: Vec<_> = natives.keys().collect();
        self.validate_headers(native_handles.as_slice());
        if !self.errors.is_empty() {
            return Err(crate::Error::multiple_by_module(self.errors));
        }
        root_module.resolve_scopes(&mut self);
        if !self.errors.is_empty() {
            return Err(crate::Error::multiple_by_module(self.errors));
        }
        let tests = if run_tests {
            root_module.take_tests()
        } else {
            vec![]
        };
        let mut database: Database = Database::new(root_module.into_definitions());
        for (_, header) in self
            .modules
            .iter()
            .filter(|(scope, _)| scope.library().is_empty())
        {
            database.apply_header(header, &natives);
        }
        let database = self
            .libraries
            .into_iter()
            .fold(database, |database, (_, library)| database.merge(library));
        let failed_tests: Vec<_> = tests
            .into_iter()
            .filter_map(|test| {
                let question = Question::new(test);
                let answers = database.unify_test(&question).collect::<Vec<_>>();
                if answers.is_empty() {
                    Some(question)
                } else {
                    None
                }
            })
            .collect();
        if failed_tests.is_empty() {
            Ok(Lumber::build(self.modules, database))
        } else {
            Err(crate::Error::test(failed_tests))
        }
    }

    fn enter_module(&mut self, module: Atom) {
        self.current_scope.push(module);
    }

    fn leave_module(&mut self) {
        self.current_scope.pop();
    }

    pub(crate) fn get_variable(&mut self, name: &str) -> Identifier {
        if let Some(existing) = self.current_environment.get(name) {
            existing.clone()
        } else {
            let ident = Identifier::new(name.to_owned());
            self.current_environment
                .insert(name.to_owned(), ident.clone());
            ident
        }
    }

    pub(crate) fn reset_environment(&mut self) {
        self.current_environment.clear();
    }

    pub(crate) fn add_module(&mut self, module: Atom) -> crate::Result<Option<Module>> {
        let scope = self.current_scope.join(module.clone());
        if self.modules.contains_key(&scope) {
            self.error_duplicate_module(scope);
            return Ok(None);
        }
        self.enter_module(module.clone());
        self.modules
            .insert(scope, ModuleHeader::new(self.current_scope.clone()));
        let mut module_path = self
            .current_scope
            .into_iter()
            .fold(self.root_path.clone(), |path, atom| {
                path.join(atom.as_ref())
            })
            .with_extension("lumber");
        if !module_path.exists() {
            module_path = module_path.with_file_name(format!("{}/mod.lumber", module.as_ref()));
        }
        let source = std::fs::read_to_string(&module_path)?;
        let module = Module::new(&source, self)?;
        self.leave_module();
        Ok(Some(module))
    }

    fn current_module_mut(&mut self) -> &mut ModuleHeader {
        self.modules.get_mut(&self.current_scope).unwrap()
    }

    pub(crate) fn declare_export(&mut self, export: Handle) {
        let export = self.current_module_mut().insert_public(export);
        if let Some(export) = export {
            self.error_duplicate_export(export);
        }
    }

    pub(crate) fn declare_operator_export(&mut self, operator: Atom) {
        let export = self.current_module_mut().insert_public_operator(operator);
        if let Some(export) = export {
            self.error_duplicate_export_op(export);
        }
    }

    pub(crate) fn declare_mutable(&mut self, handle: Handle) {
        let handle = self.current_module_mut().insert_mutable(handle);
        if let Some(handle) = handle {
            self.error_duplicate_mutable(handle);
        }
    }

    pub(crate) fn declare_alias(&mut self, alias: Handle, source: Handle) {
        let alias = self.current_module_mut().insert_alias(alias, source);
        if let Some((alias, source)) = alias {
            self.error_duplicate_import(alias, source);
        }
    }

    pub(crate) fn declare_incomplete(&mut self, handle: Handle) {
        let (export, incomplete) = self.current_module_mut().insert_incomplete(handle);
        if let Some(incomplete) = incomplete {
            self.error_duplicate_incomplete(incomplete);
        } else if let Some(export) = export {
            self.error_duplicate_export(export);
        }
    }

    pub(crate) fn import_glob(&mut self, module: Scope) {
        let module = self.current_module_mut().insert_glob(module);
        if let Some(module) = module {
            self.error_duplicate_glob(module);
        }
    }

    pub(crate) fn declare_native(&mut self, predicate: Handle) {
        let native = self.current_module_mut().insert_native(predicate);
        if let Some(native) = native {
            self.error_duplicate_native(native);
        }
    }

    pub(crate) fn declare_operator_alias(&mut self, operator: Atom, scope: Scope) {
        let scope = self
            .current_module_mut()
            .insert_operator_alias(operator.clone(), scope);
        if let Some(scope) = scope {
            self.error_duplicate_operator_alias(operator, scope);
        }
    }

    pub(crate) fn declare_operator(&mut self, operator: Operator) {
        let operator = self.current_module_mut().insert_operator(operator);
        if let Some(operator) = operator {
            self.error_duplicate_operator(operator);
        }
    }

    pub(crate) fn declare_predicate(&mut self, predicate: Handle) {
        self.current_module_mut().insert(predicate);
    }

    fn validate_headers(&mut self, natives: &[&Handle]) {
        let modules = self
            .modules
            .iter()
            .filter(|(scope, _)| scope.library().is_empty());
        for (scope, module) in modules.clone() {
            let errors = module.errors(self, natives);
            if !errors.is_empty() {
                self.errors.entry(scope.clone()).or_default().extend(errors);
            }
        }
        let mappings: HashMap<_, _> = modules
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect::<Vec<(_, _)>>()
            .into_iter()
            .map(|(scope, module)| {
                let mappings: HashMap<_, _> = module
                    .operators
                    .iter()
                    .filter_map(|(op_key, operator)| {
                        let handle = self.resolve_handle_in_scope(
                            operator.handle(),
                            &operator.handle().module(),
                        )?;
                        Some((op_key.clone(), handle))
                    })
                    .collect();
                (scope, mappings)
            })
            .collect();
        for (scope, mappings) in mappings {
            let module = self.modules.get_mut(&scope).unwrap();
            for (op_key, handle) in mappings {
                *module.operators.get_mut(&op_key).unwrap().handle_mut() = handle;
            }
        }
    }

    pub(crate) fn resolve_scopes(&mut self, module: &mut Module, name: Atom) {
        self.enter_module(name);
        module.resolve_scopes(self);
        self.leave_module();
    }

    pub(crate) fn resolve_handle<'a>(&'a mut self, handle: &'a Handle) -> Option<Handle> {
        self.resolve_handle_in_scope(handle, &self.current_scope.clone())
    }

    pub(crate) fn resolve_handle_in_scope<'a>(
        &'a mut self,
        handle: &'a Handle,
        in_scope: &Scope,
    ) -> Option<Handle> {
        if let Some(library) = handle.library().first() {
            match self.libraries.get(&library) {
                None => self.error_unlinked_library(handle, &library),
                Some(lib) if lib.exports(handle) => return Some(handle.clone()),
                Some(..) => self.error_unresolved_library_predicate(handle, &library),
            }
            return None;
        }
        let module = handle.module();
        let resolved = match self.modules.get(&module) {
            Some(module) => module.resolve(handle, in_scope, self),
            None => {
                self.error_undeclared_module(handle, module);
                return None;
            }
        };
        match resolved {
            Ok(resolved) => Some(resolved.clone()),
            Err(error) => {
                self.current_errors_mut().push(error);
                None
            }
        }
    }

    pub(crate) fn resolve_operator<'a>(&'a mut self, operator: &OpKey) -> Option<Operator> {
        self.resolve_operator_in_scope(operator, &self.current_scope.clone())
    }

    pub(crate) fn resolve_operator_in_scope<'a>(
        &'a mut self,
        operator: &OpKey,
        in_scope: &Scope,
    ) -> Option<Operator> {
        let resolved = match self.modules.get(in_scope) {
            Some(module) => module.resolve_operator(operator, in_scope, self),
            None => {
                self.error_undeclared_module_op(operator, in_scope);
                return None;
            }
        };
        match resolved {
            Ok(resolved) => Some(resolved.clone()),
            Err(error) => {
                self.current_errors_mut().push(error);
                None
            }
        }
    }
}

impl Context<'_> {
    fn current_errors_mut(&mut self) -> &mut Vec<crate::Error> {
        self.errors.entry(self.current_scope.clone()).or_default()
    }

    pub(crate) fn error_duplicate_module(&mut self, module: Scope) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Module {} declared multiple times.",
            module
        )));
    }

    pub(crate) fn error_duplicate_export(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "{} exported multiple times.",
            handle
        )));
    }

    pub(crate) fn error_duplicate_export_op(&mut self, operator: Atom) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "operator {} exported multiple times.",
            operator.as_ref()
        )));
    }

    pub(crate) fn error_duplicate_incomplete(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "{} decared as incomplete multiple times.",
            handle
        )));
    }

    pub(crate) fn error_duplicate_mutable(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "{} set as mutable multiple times.",
            handle
        )));
    }

    pub(crate) fn error_negative_scope(&mut self, span: Span) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Scope {} goes above the main module.",
            span.as_str()
        )));
    }

    pub(crate) fn error_duplicate_import(&mut self, import: Handle, from: Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "{} already imported from {}.",
            import, from
        )));
    }

    pub(crate) fn error_duplicate_operator_alias(&mut self, import: Atom, from: Scope) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "{} already imported from {}.",
            import, from
        )));
    }

    pub(crate) fn error_duplicate_glob(&mut self, module: Scope) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Module {} imported multiple times.",
            module
        )));
    }

    pub(crate) fn error_duplicate_native(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Native function {} declared multiple times.",
            handle
        )));
    }

    pub(crate) fn error_duplicate_operator(&mut self, operator: Operator) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Operator {:?} declared multiple times.",
            operator,
        )));
    }

    pub(crate) fn error_invalid_alias_arity(&mut self, input: &Handle, output: &Handle) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Cannot change arity of {} when aliasing to {}.",
            input, output,
        )));
    }

    pub(crate) fn error_singleton_variable(&mut self, handle: &Handle, variable: &str) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Singleton variable {} in predicate {}.",
            variable, handle,
        )));
    }

    pub(crate) fn error_unlinked_library(&mut self, handle: &Handle, library: &Atom) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Referencing predicate {} from unlinked library {}.",
            handle, library,
        )));
    }

    pub(crate) fn error_undeclared_module(&mut self, handle: &Handle, module: Scope) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Referencing predicate {} from undeclared module {}.",
            handle, module,
        )));
    }

    pub(crate) fn error_undeclared_module_op(&mut self, operator: &OpKey, module: &Scope) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Referencing operator {} from undeclared module {}.",
            operator, module,
        )));
    }

    pub(crate) fn error_unresolved_library_predicate(&mut self, handle: &Handle, library: &Atom) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "No predicate {} is exported by the library {}.",
            handle, library,
        )));
    }

    pub(crate) fn error_operator_precedence(&mut self, name: Atom, precedence: Int) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Precedence {} for operator {} is too high (maximum is 9).",
            precedence, name,
        )))
    }

    pub(crate) fn error_unary_operator_restriction(&mut self, name: Atom) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Operator {} is defined as a unary expression operator, so must be right associative with maximum (9) precedence.",
            name,
        )))
    }

    pub(crate) fn error_operator_arity_relation(&mut self, name: Atom, len: u32) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Arity of relational operator {} must be 1 or 2, found {}.",
            name, len,
        )))
    }

    pub(crate) fn error_operator_arity_expression(&mut self, name: Atom, len: u32) {
        self.current_errors_mut().push(crate::Error::parse(&format!(
            "Arity of expression operator {} must be 2 or 3, found {}.",
            name, len,
        )))
    }
}
