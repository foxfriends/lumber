use super::*;
use std::rc::Rc;

/// A path to a defined rule.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Scope {
    /// The library the rule is defined in, if not defined by the user.
    lib: Option<Atom>,
    /// The path to this rule, relative to the library root.
    path: Rc<Vec<Atom>>,
}
