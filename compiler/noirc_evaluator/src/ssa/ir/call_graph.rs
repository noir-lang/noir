use std::collections::BTreeSet;

use fxhash::FxHashMap as HashMap;
use petgraph::graph::{DiGraph, NodeIndex as PetGraphIndex};

use super::{
    function::{Function, FunctionId},
    instruction::Instruction,
    value::Value,
};

struct CallGraph {
    graph: DiGraph<FunctionId, ()>,
    ids_to_indices: HashMap<FunctionId, PetGraphIndex>,
    indices_to_ids: HashMap<PetGraphIndex, FunctionId>,
}

pub(crate) fn build_call_graph(
    dependencies: HashMap<FunctionId, BTreeSet<FunctionId>>,
) -> (DiGraph<FunctionId, ()>, HashMap<FunctionId, PetGraphIndex>, HashMap<PetGraphIndex, FunctionId>)
{
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

    (graph, ids_to_indices, indices_to_ids)
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
