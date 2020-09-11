use std::rc::Rc;

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Atom(Rc<String>);
