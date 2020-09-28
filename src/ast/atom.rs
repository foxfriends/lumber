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

impl Atom {
    fn from_string(string: String) -> Atom {
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

    pub fn from_str(s: &str) -> Atom {
        Self::from_string(s.to_owned())
    }

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
        Self::from_string(string)
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO: properly quote atoms as necessary
        //       somehow detect that by un-parsing the atom string?
        self.0.fmt(f)
    }
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
