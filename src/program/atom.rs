use crate::parser::Rule;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct Atomizer<'i> {
    atoms: HashMap<&'i str, Rc<String>>,
}

impl<'i> Atomizer<'i> {
    pub fn atomize(&mut self, pair: crate::Pair<'i>) -> Atom {
        assert_eq!(pair.as_rule(), Rule::atom);
        let pair = just!(pair.into_inner());
        let string = match pair.as_rule() {
            Rule::bare_atom => pair.as_str(),
            Rule::quoted_atom => {
                let atom = pair.as_str().trim_matches('#');
                &atom[1..atom.len() - 1]
            }
            _ => unreachable!(),
        };
        let rc = self
            .atoms
            .entry(string)
            .or_insert_with(|| Rc::new(string.to_owned()))
            .clone();
        Atom(rc)
    }
}

/// A meaningless, constant symbol.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Atom(Rc<String>);

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
