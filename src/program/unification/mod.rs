use crate::Binding;
use std::borrow::Cow;

mod database;
mod patterns;

type Bindings<'a> = Box<dyn Iterator<Item = Cow<'a, Binding>> + 'a>;

pub(crate) use patterns::unify_patterns;
