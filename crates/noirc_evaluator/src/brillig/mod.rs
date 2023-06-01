use std::collections::HashMap;

use self::{artefact::BrilligArtefact, brillig_gen::BrilligGen};

pub(crate) mod acvm_brillig;
pub(crate) mod artefact;
pub(crate) mod brillig_gen;

use crate::ssa_refactor::{
    ir::function::{Function, FunctionId},
    ssa_gen::Ssa,
};
/// Context struct for the brillig gen pass.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA functions to their brillig opcode
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtefact>,
}

impl Brillig {
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
    pub(crate) fn to_brillig(&self) -> Brillig {
        let mut brillig = Brillig::default();
        for f in self.functions.values() {
            let id = f.id();
            if id != self.main_id {
                brillig.compile(f);
            }
        }
        brillig
    }
}
