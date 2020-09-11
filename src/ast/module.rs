use super::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Module {
    /// The path from which to resolve dependencies of this module. If this module was read from
    /// file, this will be a path to the source file. Otherwise, if this module is from a
    /// non-filesystem location, this is simply a directory from which to search for more modules.
    path: PathBuf,
    /// The modules that we expect to be able to find relative to this module.
    mods: HashSet<Atom>,
    /// Scopes (modules) from which to find *imp*licit imports.
    imps: HashSet<Scope>,
    /// Predicates which have been imported directly from other modules.
    uses: HashSet<Handle>,
    /// Predicates and functions defined in this module which are exposed to other modules.
    pubs: HashSet<Handle>,
    /// Native predicates and functions bound to this module.
    nats: HashSet<Handle>,
    /// All predicates and functions defined in this module.
    defs: HashMap<Handle, Definition>,
}
