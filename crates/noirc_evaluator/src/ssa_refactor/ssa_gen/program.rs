use std::{collections::BTreeMap, fmt::Display};

use iter_extended::btree_map;

use crate::ssa_refactor::ir::function::{Function, FunctionId};

/// Contains the entire SSA representation of the program.
pub(crate) struct Ssa {
    pub(crate) functions: BTreeMap<FunctionId, Function>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions
    pub(crate) fn new(functions: Vec<Function>) -> Self {
        Self { functions: btree_map(functions, |f| (f.id(), f)) }
    }

    pub(crate) fn main(&self) -> &Function {
        self.functions.first_key_value().expect("Expected there to be at least 1 SSA function").1
    }
}

impl Display for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, function) in &self.functions {
            writeln!(f, "{function}")?;
        }
        Ok(())
    }
}
