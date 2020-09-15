use super::*;

/// A structured value.
#[derive(Clone, Debug)]
pub struct Struct {
    /// The tag of the struct
    name: Atom,
    /// The shape of the struct
    arity: Vec<Arity>,
    /// The values in the struct
    fields: Vec<Pattern>,
}
