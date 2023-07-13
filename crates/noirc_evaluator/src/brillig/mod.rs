pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use self::{
    brillig_gen::{brillig_fn::FunctionContext, convert_ssa_function},
    brillig_ir::artifact::{BrilligArtifact, Label},
};
use crate::ssa_refactor::{
    ir::{
        function::{Function, FunctionId, RuntimeType},
        value::Value,
    },
    ssa_gen::Ssa,
};
use std::collections::{HashMap, HashSet};

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
        // That means all the functions marked as brillig and ACIR functions called by them
        let mut reachable_function_ids: HashSet<FunctionId> = HashSet::new();

        // Initialize the queue with all the functions marked as brillig
        let mut reachability_queue: Vec<FunctionId> = self
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

        while !reachability_queue.is_empty() {
            let func = &self.functions[&reachability_queue
                .pop()
                .expect("Queue should have already been checked for emptiness")];
            reachable_function_ids.insert(func.id());
            for (_, value) in func.dfg.values_iter() {
                // All reachable functions appear as literals after defunctionalization of the SSA
                if let Value::Function(function_id) = value {
                    if !reachable_function_ids.contains(function_id)
                        && !reachability_queue.contains(function_id)
                    {
                        reachability_queue.push(*function_id);
                    }
                }
            }
        }

        let mut brillig = Brillig::default();
        for brillig_function_id in reachable_function_ids {
            let func = &self.functions[&brillig_function_id];
            brillig.compile(func);
        }

        brillig
    }
}
