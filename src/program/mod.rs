#[macro_use]
mod macros;

#[cfg(test)]
mod test;

mod alias;
mod arity;
mod atom;
mod body;
mod computation;
mod conjunction;
mod definition;
mod disjunction;
mod handle;
mod identifier;
mod implication;
mod literal;
mod module;
mod pattern;
mod program;
mod query;
mod scope;
mod r#struct;
mod unification;

pub use alias::Alias;
pub use arity::Arity;
pub use atom::Atom;
pub use body::Body;
pub use computation::Computation;
pub use conjunction::Conjunction;
pub use definition::Definition;
pub use disjunction::Disjunction;
pub use handle::Handle;
pub use identifier::Identifier;
pub use implication::Implication;
pub use literal::Literal;
pub use module::Module;
pub use pattern::Pattern;
pub use program::Program;
pub use query::Query;
pub use r#struct::Struct;
pub use scope::Scope;
pub use unification::Unification;

mod builtin;
mod context;
mod database;
mod fields;
mod module_header;
mod native_function;
mod prec_climber;

use atom::Atomizer;
use context::Context;
use database::Database;
use fields::fields;
use module_header::ModuleHeader;
use native_function::NativeFunction;
use prec_climber::{Operator, PrecClimber};
