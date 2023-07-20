pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use self::{
    brillig_gen::{brillig_fn::FunctionContext, convert_ssa_function},
    brillig_ir::artifact::{BrilligArtifact, Label},
};
use crate::ssa_refactor::{
    ir::function::{Function, FunctionId, RuntimeType},
    ssa_gen::Ssa,
};
use std::collections::HashMap;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(&mut self, func: &Function) {
        let obj = convert_ssa_function(func);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    /// Finds a brillig function artifact by its function label
    pub(crate) fn find_by_function_label(&self, function_label: Label) -> Option<&BrilligArtifact> {
        self.ssa_function_to_brillig.iter().find_map(|(function_id, obj)| {
            if FunctionContext::function_id_to_function_label(*function_id) == function_label {
                Some(obj)
            } else {
                None
            }
        })
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Compile to brillig brillig functions and ACIR functions reachable from them
    pub(crate) fn to_brillig(&self) -> Brillig {
        // Collect all the function ids that are reachable from brillig
        let functions_marked_as_brillig: Vec<FunctionId> = self
            .functions
            .iter()
            .filter_map(
                |(id, func)| {
                    if func.runtime() == RuntimeType::Brillig {
                        Some(*id)
                    } else {
                        None
                    }
                },
            )
            .collect();

        let brillig_reachable_function_ids = self.reachable_functions(&functions_marked_as_brillig);

        let mut brillig = Brillig::default();
        for brillig_function_id in brillig_reachable_function_ids {
            let func = &self.functions[&brillig_function_id];
            brillig.compile(func);
        }

        brillig
    }
}
