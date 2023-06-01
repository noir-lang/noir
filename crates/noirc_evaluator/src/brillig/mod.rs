use std::collections::HashMap;

use self::{artefact::BrilligArtefact, brillig_gen::BrilligGen};

pub(crate) mod acvm_brillig;
pub(crate) mod artefact;
pub(crate) mod brillig_gen;

use crate::ssa_refactor::{
    ir::function::{Function, FunctionId, RuntimeType},
    ssa_gen::Ssa,
};
/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA functions to their brillig opcode
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtefact>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artefacts
    pub(crate) fn compile(&mut self, func: &Function) {
        let obj = BrilligGen::compile(func);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtefact;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Generate compilation artefacts for brillig funtions
    pub(crate) fn to_brillig(&self) -> Brillig {
        let mut brillig = Brillig::default();
        for f in self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig) {
            let id = f.id();
            if id != self.main_id {
                brillig.compile(f);
            }
        }
        brillig
    }
}
