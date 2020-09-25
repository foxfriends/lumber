use super::*;
use pest::Span;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Context {
    pub root_path: PathBuf,
    pub atomizer: Atomizer,
    pub current_scope: Scope,
    pub variables: Vec<String>,
    pub current_environment: HashMap<String, usize>,
    pub modules: HashMap<Scope, ModuleHeader>,
    pub errors: HashMap<Scope, Vec<crate::Error>>,
}

impl Context {
    pub fn compile<'p>(
        root_path: PathBuf,
        source: &str,
        natives: HashMap<Handle, NativeFunction<'p>>,
    ) -> crate::Result<Program<'p>> {
        let mut context = Self {
            root_path: root_path.clone(),
            atomizer: Atomizer::default(),
            current_scope: Scope::default(),
            variables: vec![],
            current_environment: HashMap::default(),
            modules: HashMap::default(),
            errors: HashMap::default(),
        };
        context
            .modules
            .insert(Scope::default(), ModuleHeader::new(Scope::default()));

        let mut root_module = Module::new(root_path, source, &mut context)?;
        let native_handles: Vec<_> = natives.keys().collect();
        context.validate_headers(native_handles.as_slice());
        if !context.errors.is_empty() {
            return Err(crate::Error::multiple_by_module(context.errors));
        }
        root_module.resolve_scopes(&mut context);
        if !context.errors.is_empty() {
            return Err(crate::Error::multiple_by_module(context.errors));
        }
        let mut database: Database = root_module.into_definitions().collect();
        for header in context.modules.values() {
            database.apply_header(header);
        }
        Ok(Program::build(database))
    }

    fn enter_module(&mut self, module: Atom) {
        self.current_scope.push(module);
    }

    fn leave_module(&mut self) {
        self.current_scope.pop();
    }

    pub fn get_variable(&mut self, name: &str) -> Identifier {
        if let Some(existing) = self.current_environment.get(name) {
            Identifier::new(*existing)
        } else {
            let index = self.variables.len();
            self.variables.push(name.to_owned());
            self.current_environment.insert(name.to_owned(), index);
            Identifier::new(index)
        }
    }

    pub fn fresh_variable(&mut self) -> Identifier {
        self.get_variable(&format!("#{}", self.variables.len()))
    }

    pub fn reset_environment(&mut self) {
        self.current_environment.clear();
    }

    pub fn add_module(&mut self, module: Atom) -> crate::Result<Option<Module>> {
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
        let module = Module::new(module_path, &source, self)?;
        self.leave_module();
        Ok(Some(module))
    }

    fn current_module_mut(&mut self) -> &mut ModuleHeader {
        self.modules.get_mut(&self.current_scope).unwrap()
    }

    pub fn declare_export(&mut self, export: Handle) {
        let export = self.current_module_mut().insert_public(export);
        if let Some(export) = export {
            self.error_duplicate_export(export);
        }
    }

    pub fn declare_mutable(&mut self, handle: Handle) {
        let handle = self.current_module_mut().insert_mutable(handle);
        if let Some(handle) = handle {
            self.error_duplicate_mutable(handle);
        }
    }

    pub fn declare_alias(&mut self, alias: Handle, source: Handle) {
        let alias = self.current_module_mut().insert_alias(alias, source);
        if let Some((alias, source)) = alias {
            self.error_duplicate_import(alias, source);
        }
    }

    pub fn declare_incomplete(&mut self, handle: Handle) {
        let (export, incomplete) = self.current_module_mut().insert_incomplete(handle);
        if let Some(incomplete) = incomplete {
            self.error_duplicate_incomplete(incomplete);
        } else if let Some(export) = export {
            self.error_duplicate_export(export);
        }
    }

    pub fn import_glob(&mut self, module: Scope) {
        let module = self.current_module_mut().insert_glob(module);
        if let Some(module) = module {
            self.error_duplicate_glob(module);
        }
    }

    pub fn declare_native(&mut self, predicate: Handle) {
        let native = self.current_module_mut().insert_native(predicate);
        if let Some(native) = native {
            self.error_duplicate_native(native);
        }
    }

    pub fn declare_predicate(&mut self, predicate: Handle) {
        self.current_module_mut().insert(predicate);
    }

    fn validate_headers(&mut self, natives: &[&Handle]) {
        for (scope, module) in &self.modules {
            let errors = module.errors(self, natives);
            if !errors.is_empty() {
                self.errors.entry(scope.clone()).or_default().extend(errors);
            }
        }
    }

    pub fn resolve_scopes(&mut self, module: &mut Module, name: Atom) {
        self.enter_module(name);
        module.resolve_scopes(self);
        self.leave_module();
    }

    pub fn resolve_handle<'a>(&'a mut self, handle: &'a Handle) -> Option<Handle> {
        self.resolve_handle_in_scope(handle, &self.current_scope.clone())
    }

    pub fn resolve_handle_in_scope<'a>(
        &'a mut self,
        handle: &'a Handle,
        in_scope: &Scope,
    ) -> Option<Handle> {
        let module = handle.module();
        let resolved = self
            .modules
            .get(&module)
            .unwrap()
            .resolve(handle, in_scope, self);
        match resolved {
            Ok(resolved) => return Some(resolved.clone()),
            Err(error) => {
                self.current_errors_mut().push(error);
                None
            }
        }
    }
}

impl Context {
    fn current_errors_mut(&mut self) -> &mut Vec<crate::Error> {
        self.errors.entry(self.current_scope.clone()).or_default()
    }

    pub fn error_duplicate_module(&mut self, module: Scope) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Module {} declared multiple times.",
            module
        )));
    }

    pub fn error_duplicate_export(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "{} exported multiple times.",
            handle
        )));
    }

    pub fn error_duplicate_incomplete(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "{} decared as incomplete multiple times.",
            handle
        )));
    }

    pub fn error_duplicate_mutable(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "{} set as mutable multiple times.",
            handle
        )));
    }

    pub fn error_negative_scope(&mut self, span: Span) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Scope {} goes above the main module.",
            span.as_str()
        )));
    }

    pub fn error_duplicate_import(&mut self, import: Handle, from: Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "{} already imported from {}.",
            import, from
        )));
    }

    pub fn error_duplicate_glob(&mut self, module: Scope) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Module {} imported multiple times.",
            module
        )));
    }

    pub fn error_duplicate_native(&mut self, handle: Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Native function {} declared multiple times.",
            handle
        )));
    }

    pub fn error_unrecognized_operator(&mut self, token: &str) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Unrecognized operator `{}`.",
            token
        )));
    }

    pub fn error_unresolved_handle(&mut self, handle: &Handle) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Unresolved predicate {}.",
            handle
        )));
    }

    pub fn error_ambiguous_reference(&mut self, handle: &Handle, candidates: Vec<Handle>) {
        self.current_errors_mut().push(crate::Error::parse(format!(
            "Ambiguous reference {}. Could be referring to any of:\n{}",
            handle,
            candidates
                .iter()
                .map(|candidate| format!("\t{}", candidate))
                .collect::<Vec<_>>()
                .join("\n"),
        )));
    }
}
