use crate::parser::Rule;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::{Rc, Weak};
use weak_table::WeakHashSet;

thread_local! {
    static ATOMS: RefCell<WeakHashSet<Weak<String>>> = Default::default();
}

/// A meaningless, constant symbol.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) struct Atom(Rc<String>);

impl From<String> for Atom {
    fn from(string: String) -> Self {
        if string.is_empty() {
            // TODO: should this really be a panic?
            panic!("an atom cannot be empty");
        }
        ATOMS.with(|atoms| {
            let mut atoms = atoms.borrow_mut();
            if let Some(existing) = atoms.get(&string) {
                Atom(existing)
            } else {
                let rc = Rc::new(string);
                atoms.insert(rc.clone());
                Atom(rc)
            }
        })
    }
}

impl From<&str> for Atom {
    fn from(string: &str) -> Self {
        Self::from(string.to_owned())
    }
}

impl Atom {
    pub fn new(pair: crate::Pair) -> Atom {
        assert_eq!(pair.as_rule(), Rule::atom);
        let pair = just!(pair.into_inner());
        let string = match pair.as_rule() {
            Rule::bare_atom => pair.as_str().to_owned(),
            Rule::quoted_atom => {
                let atom = pair.as_str().trim_matches('#');
                atom[1..atom.len() - 1].to_owned()
            }
            _ => unreachable!(),
        };
        Self::from(string)
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO: properly quote atoms as necessary
        //       somehow detect that by un-parsing the atom string?
        if self
            .0
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            && self.0.chars().next().unwrap().is_ascii_alphabetic()
        {
            self.0.fmt(f)
        } else {
            if !self.0.contains('\'') {
                write!(f, "'{}'", self.0)
            } else {
                let n = self
                    .0
                    .chars()
                    .fold((0, None), |(max, len), ch| match len {
                        None if ch == '\'' => (max, Some(0)),
                        Some(n) if ch == '#' => (usize::max(max, n + 1), Some(n + 1)),
                        _ => (max, None),
                    })
                    .0
                    + 1;
                write!(f, "{}'{}'{}", "#".repeat(n), self.0, "#".repeat(n))
            }
        }
    }
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
