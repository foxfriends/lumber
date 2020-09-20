use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

// TODO: figure out the parameter/return type of this function
#[derive(Clone)]
pub struct NativeFunction {
    function: Rc<Box<dyn Fn()>>,
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "NativeFunction {{ function: {:p} }}", self.function)
    }
}
