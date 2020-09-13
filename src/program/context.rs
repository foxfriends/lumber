use super::*;
use pest::Span;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Context<'i> {
    pub root_path: PathBuf,
    pub atomizer: Atomizer<'i>,
    pub current_scope: Scope,
    pub modules: HashMap<Scope, ModuleHeader>,
    pub errors: HashMap<Scope, Vec<crate::Error>>,
}

impl Context<'_> {
    pub fn compile(root_path: PathBuf, source: &str) -> crate::Result<Program> {
        let mut context = Self {
            root_path,
            atomizer: Atomizer::default(),
            current_scope: Scope::default(),
            modules: HashMap::default(),
            errors: HashMap::default(),
        };
        context
            .modules
            .insert(Scope::default(), ModuleHeader::default());
        context.finish()
    }

    fn enter_module(&mut self, module: Atom) {
        self.current_scope.push(module);
    }

    fn leave_module(&mut self) {
        self.current_scope.pop();
    }

    pub fn declare_module(&mut self, module: Atom) {
        let scope = self.current_scope.join(module);
        if self.modules.contains_key(&scope) {
            self.error_duplicate_module(scope);
        } else {
            self.modules.insert(scope, ModuleHeader::default());
        }
    }

    pub fn declare_export(&mut self, export: Handle) {
        let export = self
            .modules
            .get_mut(&self.current_scope)
            .unwrap()
            .insert_public(export);
        if let Some(export) = export {
            self.error_duplicate_export(export);
        }
    }

    pub fn declare_predicate(&mut self, predicate: Handle) {
        self.modules
            .get_mut(&self.current_scope)
            .unwrap()
            .insert(predicate);
    }

    fn finish(self) -> crate::Result<Program> {
        todo!();
    }
}

impl<'i> Context<'i> {
    fn current_errors(&mut self) -> &mut Vec<crate::Error> {
        self.errors.entry(self.current_scope.clone()).or_default()
    }

    pub fn error_duplicate_module(&mut self, module: Scope) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("Module {} declared multiple times", module),
            source: None,
        });
    }

    pub fn error_duplicate_export(&mut self, handle: Handle) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("{} exported multiple times", handle),
            source: None,
        });
    }

    pub fn error_negative_scope(&mut self, span: Span<'i>) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("Scope {} goes above the main module.", span.as_str()),
            source: None,
        });
    }

    pub fn error_duplicate_import(&mut self, import: &Handle, from: &Handle) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("{} already imported from {}", import, from),
            source: None,
        });
    }

    pub fn error_duplicate_glob(&mut self, module: Scope) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("Module {} imported multiple times", module),
            source: None,
        });
    }

    pub fn error_duplicate_native(&mut self, handle: Handle) {
        self.current_errors().push(crate::Error {
            kind: crate::ErrorKind::Parse,
            message: format!("Native function {} declared multiple times", handle),
            source: None,
        });
    }
}
