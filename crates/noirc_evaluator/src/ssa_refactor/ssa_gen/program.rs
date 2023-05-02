use std::fmt::Display;

use crate::ssa_refactor::ir::function::Function;

/// Contains the entire SSA representation of the program.
pub struct Ssa {
    functions: Vec<Function>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions
    pub fn new(functions: Vec<Function>) -> Self {
        Self { functions }
    }
}

impl Display for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in &self.functions {
            writeln!(f, "{function}")?;
        }
        Ok(())
    }
}
