pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use self::{brillig_gen::convert_ssa_function, brillig_ir::artifact::BrilligArtifact};
use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType},
    },
    ssa_gen::Ssa,
};
use std::collections::HashMap;

pub(crate) type FuncIdEntryBlockId = HashMap<FunctionId, BasicBlockId>;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function IDs to their entry block IDs
    ///
    /// Used for external call instructions
    ssa_function_id_to_block_id: HashMap<FunctionId, BasicBlockId>,
    /// Maps SSA functions to their brillig opcode
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact>,
}

impl Brillig {
    /// Creates a Brillig object with a prefilled map of function IDs to entry block IDs
    pub(crate) fn new(ssa_function_id_to_block_id: FuncIdEntryBlockId) -> Brillig {
        Brillig { ssa_function_id_to_block_id, ssa_function_to_brillig: HashMap::new() }
    }

    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(&mut self, func: &Function) {
        let obj = convert_ssa_function(func, &self.ssa_function_id_to_block_id);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Generate compilation artifacts for brillig functions
    pub(crate) fn to_brillig(&self) -> Brillig {
        // Collect all of the brillig functions
        let brillig_functions =
            self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig);

        // Collect the entry block IDs for each function.
        //
        // Call instructions only specify their function ID, not their entry block ID.
        // But in order to jump to a function, we need to know the label that is assigned to
        // the entry-block of the function. This will be the function_id
        // concatenated with the entry_block_id.
        let brillig_func_ids_to_entry_block_ids: HashMap<FunctionId, BasicBlockId> =
            brillig_functions
                .clone()
                .map(|func| {
                    let func_id = func.id();
                    let entry_block_id = func.entry_block();
                    (func_id, entry_block_id)
                })
                .collect();

        let mut brillig = Brillig::new(brillig_func_ids_to_entry_block_ids);
        for brillig_function in brillig_functions {
            // TODO: document why we are skipping the `main_id` for Brillig functions
            if brillig_function.id() != self.main_id {
                brillig.compile(brillig_function);
            }
        }

        brillig
    }
}
