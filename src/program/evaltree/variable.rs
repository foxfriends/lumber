use super::super::evaltree::Identifier;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Variable {
    identifier: Identifier,
    generation: Option<usize>,
}

impl Variable {
    pub fn new(identifier: Identifier, generation: usize) -> Self {
        Self {
            identifier,
            generation: Some(generation),
        }
    }

    pub fn new_generationless(identifier: Identifier) -> Self {
        Self {
            identifier,
            generation: None,
        }
    }

    pub fn set_current(&self, now: Option<usize>) -> Self {
        Self {
            identifier: self.identifier.clone(),
            generation: self.generation.or(now),
        }
    }

    pub fn name(&self) -> &str {
        self.identifier.name()
    }

    pub fn generation(&self) -> Option<usize> {
        self.generation
    }

    pub fn is_wildcard(&self) -> bool {
        self.identifier.is_wildcard()
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.generation {
            Some(gen) => write!(f, "{}@{}", self.identifier, gen),
            None => write!(f, "{}@_", self.identifier),
        }
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.generation == other.generation {
            self.identifier.cmp(&other.identifier)
        } else if self.generation.is_none() {
            Ordering::Greater
        } else if other.generation.is_none() {
            Ordering::Less
        } else {
            self.generation.unwrap().cmp(&other.generation.unwrap())
        }
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
