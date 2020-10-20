use super::Binding;

/// Describes how to convert a binding of variables into a more structured form.
///
/// Currently this is implemented manually, but eventually will be derivable.
pub trait FromBinding: Sized {
    /// Constructs a value of this type from a binding of variables. If the construction
    /// cannot be completed, the full, unmodified binding should be returned as an `Err`.
    fn from_binding(binding: Binding) -> Result<Self, Binding>;
}
