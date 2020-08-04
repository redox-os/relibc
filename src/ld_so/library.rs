use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use super::linker::Symbol;

#[derive(Default, Debug)]
pub struct DepTree {
    pub name: String,
    pub deps: Vec<DepTree>,
}

impl DepTree {
    pub fn new(name: String) -> DepTree {
        DepTree {
            name,
            deps: Vec::new(),
        }
    }
}

/// Use to represnt a library as well as all th symbols that is loaded withen it.
#[derive(Default)]
pub struct Library {
    /// Global symbols
    pub globals: BTreeMap<String, Symbol>,
    /// Weak symbols
    pub weak_syms: BTreeMap<String, Symbol>,
    /// Loaded library raw data
    pub objects: BTreeMap<String, Box<[u8]>>,
    /// Loaded library in-memory data
    pub mmaps: BTreeMap<String, (usize, &'static mut [u8])>,
    /// Each object will have its children called once with no repetition.
    pub dep_tree: DepTree,
    /// A set used to detect circular dependencies in the Linker::load function
    pub cir_dep: BTreeSet<String>,
}
impl Library {
    pub fn new() -> Library {
        Default::default()
    }
    pub fn get_sym(&self, name: &str) -> Option<Symbol> {
        if let Some(value) = self.globals.get(name) {
            Some(*value)
        } else if let Some(value) = self.weak_syms.get(name) {
            Some(*value)
        } else {
            None
        }
    }
}
