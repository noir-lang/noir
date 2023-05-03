use std::{collections::BTreeMap, fmt::Display};

use iter_extended::btree_map;

use crate::ssa_refactor::ir::function::{Function, FunctionId};

/// Contains the entire SSA representation of the program.
///
/// It is expected that the main function is always the first
/// function in the functions vector.
pub struct Ssa {
    pub functions: BTreeMap<FunctionId, Function>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions
    pub fn new(functions: Vec<Function>) -> Self {
        Self { functions: btree_map(functions, |f| (f.id(), f)) }
    }

    pub fn main(&mut self) -> &mut Function {
        self.functions
            .first_entry()
            .expect("Expected there to be at least 1 SSA function")
            .into_mut()
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
