use super::*;
use std::rc::Rc;

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Scope {
    lib: Option<Atom>,
    path: Rc<Vec<Atom>>,
}
