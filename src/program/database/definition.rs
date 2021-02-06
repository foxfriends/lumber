use super::super::evaltree::*;
use super::super::NativeFunction;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub(crate) enum DatabaseDefinition<'p> {
    Static(Definition),
    Mutable(RefCell<Definition>),
    Alias(Handle),
    Native(NativeFunction<'p>),
}

impl DatabaseDefinition<'_> {
    pub(super) fn set_mutable(&mut self) {
        match self {
            Self::Static(def) => *self = Self::Mutable(RefCell::new(std::mem::take(def))),
            _ => panic!("Cannot change definition to mutable"),
        }
    }

    pub fn handles_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Handle> + 'a> {
        match self {
            Self::Static(def) => Box::new(def.bodies_mut().flat_map(|body| body.handles_mut())),
            Self::Mutable(def) => Box::new(
                def.get_mut()
                    .bodies_mut()
                    .flat_map(|body| body.handles_mut()),
            ),
            Self::Alias(handle) => Box::new(std::iter::once(handle)),
            _ => Box::new(std::iter::empty()),
        }
    }
}
