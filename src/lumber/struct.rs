use super::Value;
use crate::ast::*;
use std::fmt::{self, Display, Formatter};

/// A Lumber structure, containing a combination of named and indexed fields. Atoms in Lumber are
/// the same as structs with no fields.
#[derive(Clone, PartialEq, Debug)]
pub struct Struct {
    pub(crate) name: Atom,
    pub(crate) contents: Option<Box<Option<Value>>>,
}

impl Struct {
    pub(crate) fn raw(name: Atom, contents: Option<Box<Option<Value>>>) -> Self {
        Self { name, contents }
    }

    /// Constructs a structure, with a (possibly unknown) value inside.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let structure = Struct::new("hello", None);
    /// assert!(!structure.is_atom());
    /// ```
    #[inline(always)]
    pub fn new(name: impl Into<String>, contents: Option<Value>) -> Self {
        Self::raw(Atom::from(name.into()), Some(Box::new(contents)))
    }

    /// Constructs an atom.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert!(atom.is_atom());
    /// ```
    #[inline(always)]
    pub fn atom(name: impl Into<String>) -> Self {
        Self::raw(Atom::from(name.into()), None)
    }

    /// Checks if this struct is actually just an atom. An atom is a struct with no fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert!(atom.is_atom());
    /// ```
    pub fn is_atom(&self) -> bool {
        self.contents.is_none()
    }

    /// Gets this struct's value if it is an atom, otherwise, returns `None`.
    pub fn as_atom(&self) -> Option<&str> {
        if self.is_atom() {
            Some(self.name())
        } else {
            None
        }
    }

    /// The name or symbol of this struct or atom.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::Struct;
    /// let atom = Struct::atom("hello");
    /// assert_eq!(atom.name(), "hello");
    /// ```
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// The contents of this struct, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::{Value, Struct};
    /// let structure = Struct::new("hello", Some(Value::from("world")));
    /// assert_eq!(structure.contents().unwrap(), &Some(Value::string("world")));
    /// ```
    pub fn contents(&self) -> Option<&Option<Value>> {
        self.contents.as_deref()
    }

    /// The contents of this struct, if any, accessed mutably.
    pub fn contents_mut(&mut self) -> Option<&mut Option<Value>> {
        self.contents.as_deref_mut()
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)?;
        if let Some(contents) = &self.contents {
            match contents.as_ref() {
                Some(contents) => write!(f, "({})", contents)?,
                None => write!(f, "(_)")?,
            }
        }
        Ok(())
    }
}
