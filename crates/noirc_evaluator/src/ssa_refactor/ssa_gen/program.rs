use std::{collections::BTreeMap, fmt::Display};

use iter_extended::btree_map;

use crate::ssa_refactor::ir::{
    function::{Function, FunctionId},
    map::AtomicCounter,
};

/// Contains the entire SSA representation of the program.
pub(crate) struct Ssa {
    pub(crate) functions: BTreeMap<FunctionId, Function>,
    pub(crate) main_id: FunctionId,
    pub(crate) next_id: AtomicCounter<Function>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions.
    /// The first function in this vector is expected to be the main function.
    pub(crate) fn new(functions: Vec<Function>) -> Self {
        let main_id = functions.first().expect("Expected at least 1 SSA function").id();
        let mut max_id = main_id;

        let functions = btree_map(functions, |f| {
            max_id = std::cmp::max(max_id, f.id());
            (f.id(), f)
        });

        Self { functions, main_id, next_id: AtomicCounter::starting_after(max_id) }
    }

    /// Returns the entry-point function of the program
    pub(crate) fn main(&self) -> &Function {
        &self.functions[&self.main_id]
    }

    /// Returns the entry-point function of the program as a mutable reference
    pub(crate) fn main_mut(&mut self) -> &mut Function {
        self.functions.get_mut(&self.main_id).expect("ICE: Ssa should have a main function")
    }
}

impl Display for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in self.functions.values() {
            writeln!(f, "{function}")?;
        }
        Ok(())
    }
}
