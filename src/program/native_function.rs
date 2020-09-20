use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

// TODO: figure out the parameter/return type of this function
#[derive(Clone)]
pub struct NativeFunction<'p> {
    function: Rc<Box<dyn Fn() + 'p>>,
}

impl<'p> NativeFunction<'p> {
    pub(crate) fn new<F>(function: F) -> Self
    where
        F: Fn() + 'p,
    {
        Self {
            function: Rc::new(Box::new(function)),
        }
    }
}

impl Debug for NativeFunction<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "NativeFunction {{ function: {:p} }}", self.function)
    }
}
