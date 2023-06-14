pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use self::{brillig_gen::BrilligGen, brillig_ir::artifact::BrilligArtifact};
use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType},
    },
    ssa_gen::Ssa,
};
use std::collections::HashMap;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA functions to their brillig opcode
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact>,
    /// Maps SSA functions to their entry block
    ssa_function_to_block: HashMap<FunctionId, BasicBlockId>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(&mut self, func: &Function) {
        let obj = BrilligGen::compile(func, self);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    pub(crate) fn function_label(&self, id: FunctionId) -> String {
        id.to_string()+"-"+ &self.ssa_function_to_block[&id].to_string()
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
        let mut brillig = Brillig::default();
        for f in self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig) {
            brillig.ssa_function_to_block.insert(f.id(), f.entry_block());
        }
        
        for f in self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig) {
            let id = f.id();
            if id != self.main_id {
                brillig.compile(f);
            }
        }
        brillig
    }
}
