use std::collections::BTreeSet;

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use petgraph::{
    algo::kosaraju_scc,
    graph::{DiGraph, NodeIndex as PetGraphIndex},
};

use crate::ssa::ssa_gen::Ssa;

use super::{
    function::{Function, FunctionId},
    instruction::Instruction,
    value::Value,
};

pub(crate) struct CallGraph {
    graph: DiGraph<FunctionId, ()>,
    ids_to_indices: HashMap<FunctionId, PetGraphIndex>,
    indices_to_ids: HashMap<PetGraphIndex, FunctionId>,
}

impl CallGraph {
    pub(crate) fn new_from_ssa(ssa: &Ssa) -> Self {
        let function_deps = ssa
            .functions
            .iter()
            .map(|(id, func)| {
                let called_functions = called_functions(func);
                (*id, called_functions)
            })
            .collect();

        Self::new_from_deps(function_deps)
    }

    pub(crate) fn new_from_deps(dependencies: HashMap<FunctionId, BTreeSet<FunctionId>>) -> Self {
        let mut graph = DiGraph::new();
        let mut ids_to_indices = HashMap::default();
        let mut indices_to_ids = HashMap::default();

        for function in dependencies.keys() {
            let index = graph.add_node(*function);
            ids_to_indices.insert(*function, index);
            indices_to_ids.insert(index, *function);
        }

        // Create edges from caller -> called
        for (function, dependencies) in dependencies {
            let function_index = ids_to_indices[&function];

            for dependency in dependencies {
                let dependency_index = ids_to_indices[&dependency];
                graph.add_edge(function_index, dependency_index, ());
            }
        }

        Self { graph, ids_to_indices, indices_to_ids }
    }

    pub(crate) fn get_recursive_functions(&self) -> HashSet<FunctionId> {
        let mut recursive_functions = HashSet::default();

        let sccs = kosaraju_scc(&self.graph);
        for scc in sccs {
            if scc.len() > 1 {
                // Mutual recursion
                for idx in scc {
                    recursive_functions.insert(self.indices_to_ids[&idx]);
                }
            } else {
                // Check for self-recursion
                let idx = scc[0];
                if self.graph.neighbors(idx).any(|n| n == idx) {
                    recursive_functions.insert(self.indices_to_ids[&idx]);
                }
            }
        }
        recursive_functions
    }

    pub(crate) fn graph(&self) -> &DiGraph<FunctionId, ()> {
        &self.graph
    }

    pub(crate) fn ids_to_indices(&self) -> &HashMap<FunctionId, PetGraphIndex> {
        &self.ids_to_indices
    }

    pub(crate) fn indices_to_ids(&self) -> &HashMap<PetGraphIndex, FunctionId> {
        &self.indices_to_ids
    }
}

/// Utility function to find out the direct calls of a function.
///
/// Returns the function IDs from all `Call` instructions without deduplication.
pub(crate) fn called_functions_vec(func: &Function) -> Vec<FunctionId> {
    let mut called_function_ids = Vec::new();
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            let Instruction::Call { func: called_value_id, .. } = &func.dfg[*instruction_id] else {
                continue;
            };

            if let Value::Function(function_id) = func.dfg[*called_value_id] {
                called_function_ids.push(function_id);
            }
        }
    }

    called_function_ids
}

/// Utility function to find out the deduplicated direct calls made from a function.
pub(crate) fn called_functions(func: &Function) -> BTreeSet<FunctionId> {
    called_functions_vec(func).into_iter().collect()
}
