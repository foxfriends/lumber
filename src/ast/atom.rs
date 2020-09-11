use std::rc::Rc;

/// A meaningless, constant symbol.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Atom(Rc<String>);
