use crate::parser::Rule;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct Atomizer {
    atoms: HashSet<Rc<String>>,
}

impl Atomizer {
    fn atomize_string(&mut self, string: String) -> Atom {
        if let Some(existing) = self.atoms.get(&string) {
            Atom(existing.clone())
        } else {
            let rc = Rc::new(string);
            self.atoms.insert(rc.clone());
            Atom(rc)
        }
    }

    pub fn atomize_str(&mut self, s: &str) -> Atom {
        self.atomize_string(s.to_owned())
    }

    pub fn atomize(&mut self, pair: crate::Pair) -> Atom {
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
        self.atomize_string(string)
    }
}

/// A meaningless, constant symbol.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) struct Atom(Rc<String>);

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
