//! Internal representations of all components of a Lumber source file/program.
#[macro_use]
mod macros;

mod alias;
mod arity;
mod atom;
mod body;
mod conjunction;
mod definition;
mod disjunction;
mod expression;
mod fields;
mod handle;
mod head;
mod identifier;
mod literal;
mod module;
mod op_arity;
mod op_key;
mod operator;
mod pattern;
mod procession;
mod query;
mod rule_kind;
mod scope;
mod step;
mod r#struct;
mod term;

pub(crate) use alias::Alias;
pub(crate) use arity::Arity;
pub(crate) use atom::Atom;
pub(crate) use body::Body;
pub(crate) use conjunction::Conjunction;
pub(crate) use definition::Definition;
pub(crate) use disjunction::Disjunction;
pub(crate) use expression::Expression;
pub(crate) use fields::Fields;
pub(crate) use handle::{AsHandle, Handle};
pub(crate) use head::Head;
pub(crate) use identifier::Identifier;
pub(crate) use literal::Literal;
pub(crate) use module::Module;
pub(crate) use op_arity::OpArity;
pub(crate) use op_key::OpKey;
pub(crate) use operator::Operator;
pub(crate) use pattern::Pattern;
pub(crate) use procession::Procession;
pub(crate) use query::Query;
pub(crate) use r#struct::Struct;
pub(crate) use rule_kind::RuleKind;
pub(crate) use scope::Scope;
pub(crate) use step::Step;
pub(crate) use term::Term;

mod context;
mod module_header;

pub(crate) use context::Context;
pub(crate) use module_header::ModuleHeader;

#[cfg(test)]
mod test;
