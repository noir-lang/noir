//! Call graph analysis
//!
//! This module provides a `CallGraph` structure that builds a directed call graph from SSA functions.
//!
//! It enables:
//! - Construction of the call graph from SSA or explicit function dependencies
//! - Detection of directly and mutually recursive functions using Kosaraju's algorithm for finding SCCs
//!
//! This utility is used by SSA passes such as inlining, which need to avoid recursive functions,
//! and purity analysis which needs to unify the purities of all functions called within another function.
use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::kosaraju_scc,
    graph::{DiGraph, NodeIndex as PetGraphIndex},
    visit::{Dfs, EdgeRef, Walker},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ssa_gen::Ssa;

use super::{
    function::{Function, FunctionId},
    instruction::Instruction,
    value::Value,
};

/// Represents a function call graph built from the SSA
/// Internally, this is a directed graph where each node is a [FunctionId] and each edge
/// represents a call from one function to another.
/// Each edge has an associated weight representing the number of times a descendant node
/// was called by an ancestor node.
pub(crate) struct CallGraph {
    graph: DiGraph<FunctionId, usize>,
    ids_to_indices: HashMap<FunctionId, PetGraphIndex>,
    indices_to_ids: HashMap<PetGraphIndex, FunctionId>,
}

impl CallGraph {
    /// Construct a [CallGraph] from the [Ssa]
    /// The edges in this graph are unweighted. Thus, it is a pure dependency map.
    pub(crate) fn from_ssa(ssa: &Ssa) -> Self {
        let function_deps = ssa
            .functions
            .iter()
            .map(|(id, func)| {
                let called_functions = called_functions(func);
                (*id, called_functions)
            })
            .collect();
        Self::from_deps(function_deps)
    }

    /// Construct a [CallGraph] from the [Ssa] with its edges weighted.
    /// An edges weight refers to the numbers of times a descendant node was called by its ancestor.
    pub(crate) fn from_ssa_weighted(ssa: &Ssa) -> Self {
        let dependencies = compute_callees(ssa);
        Self::from_deps_weighted(dependencies)
    }

    fn from_deps_weighted(dependencies: BTreeMap<FunctionId, BTreeMap<FunctionId, usize>>) -> Self {
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
            for (callee, weight) in dependencies {
                let dependency_index = ids_to_indices[&callee];
                graph.add_edge(function_index, dependency_index, weight);
            }
        }

        Self { graph, ids_to_indices, indices_to_ids }
    }

    /// Construct a [CallGraph] from an explicit dependency mapping of (caller -> callees)
    /// The edges in this graph all have a weight of one. Thus, it is a pure dependency graph.
    pub(crate) fn from_deps(dependencies: HashMap<FunctionId, BTreeSet<FunctionId>>) -> Self {
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
                graph.add_edge(function_index, dependency_index, 1);
            }
        }

        Self { graph, ids_to_indices, indices_to_ids }
    }

    /// Returns the set of all recursive functions.
    ///
    /// A function is considered recursive if:
    /// - It is self-recursive (calls itself), or
    /// - It is part of a mutual recursion cycle with other functions.
    pub(crate) fn get_recursive_functions(&self) -> HashSet<FunctionId> {
        let (_, recursive_functions) = self.sccs();
        recursive_functions
    }

    /// Returns both the call graph's strongly connected components (SCCs)
    /// as well as a utility set of all recursive functions.
    pub(crate) fn sccs(&self) -> (Vec<Vec<FunctionId>>, HashSet<FunctionId>) {
        let mut sccs_ids = Vec::new();
        let mut recursive_functions = HashSet::default();

        let sccs = kosaraju_scc(&self.graph);
        for scc in sccs {
            let scc_ids: Vec<_> = scc.iter().map(|idx| self.indices_to_ids[idx]).collect();
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
            sccs_ids.push(scc_ids);
        }
        (sccs_ids, recursive_functions)
    }

    pub(crate) fn graph(&self) -> &DiGraph<FunctionId, usize> {
        &self.graph
    }

    pub(crate) fn ids_to_indices(&self) -> &HashMap<FunctionId, PetGraphIndex> {
        &self.ids_to_indices
    }

    pub(crate) fn indices_to_ids(&self) -> &HashMap<PetGraphIndex, FunctionId> {
        &self.indices_to_ids
    }

    pub(crate) fn build_acyclic_subgraph(
        &self,
        recursive_functions: &HashSet<FunctionId>,
    ) -> CallGraph {
        let mut graph = DiGraph::new();
        let mut ids_to_indices = HashMap::default();
        let mut indices_to_ids = HashMap::default();

        // Add all non-recursive nodes
        for (&function, _) in self.ids_to_indices.iter() {
            if recursive_functions.contains(&function) {
                continue;
            }
            let index = graph.add_node(function);
            ids_to_indices.insert(function, index);
            indices_to_ids.insert(index, function);
        }

        // Create edges from caller -> called between non-recursive nodes
        for (&func_id, &old_index) in self.ids_to_indices.iter() {
            if recursive_functions.contains(&func_id) {
                continue;
            }
            let new_src = ids_to_indices[&func_id];
            for edge in self.graph.edges(old_index) {
                let dst_index = edge.target();
                let callee_id = self.indices_to_ids[&dst_index];
                if recursive_functions.contains(&callee_id) {
                    continue;
                }
                if let Some(&new_dst) = ids_to_indices.get(&callee_id) {
                    graph.add_edge(new_src, new_dst, *edge.weight());
                }
            }
        }

        Self { graph, ids_to_indices, indices_to_ids }
    }

    /// Fetch for each function the set of functions called by it, and how many times it does so.
    pub(crate) fn callees(&self) -> BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> {
        let mut callees: BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> = BTreeMap::new();

        for (caller_id, caller_index) in self.ids_to_indices.iter() {
            // Ensure all entries exist, even if they have no callers
            let entry = callees.entry(*caller_id).or_default();

            for edge in self.graph.edges(*caller_index) {
                let callee_index = edge.target();
                let callee_id = self.indices_to_ids[&callee_index];
                entry.insert(callee_id, *edge.weight());
            }
        }

        callees
    }

    /// Fetch for each function the set of functions that call it, and how many times they do so.
    pub(crate) fn callers(&self) -> BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> {
        let mut callers: BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> = BTreeMap::new();

        for (caller_id, caller_index) in self.ids_to_indices.iter() {
            // Ensure all entries exist, even if they have no callers
            callers.entry(*caller_id).or_default();

            for edge in self.graph.edges(*caller_index) {
                let callee_idx = edge.target();
                let callee_id = self.indices_to_ids[&callee_idx];
                callers.entry(callee_id).or_default().insert(*caller_id, *edge.weight());
            }
        }

        callers
    }

    /// Compute the times each function is called from any other function.
    pub(crate) fn times_called(&self) -> im::HashMap<FunctionId, usize> {
        let mut counts = im::HashMap::default();

        for edge in self.graph.edge_references() {
            let callee_idx = edge.target();
            let callee_id = self.indices_to_ids[&callee_idx];
            *counts.entry(callee_id).or_default() += *edge.weight();
        }

        // Ensure all nodes appear even if they are never called
        for function_id in self.ids_to_indices.keys() {
            counts.entry(*function_id).or_insert(0);
        }

        counts
    }

    /// Returns all functions reachable from the provided root(s).
    ///
    /// This function uses DFS internally to find all nodes reachable from the provided root(s).
    pub(crate) fn reachable_from(
        &self,
        roots: impl IntoIterator<Item = FunctionId>,
    ) -> HashSet<FunctionId> {
        let mut reachable = HashSet::default();

        for root in roots {
            // If the root does not exist, skip it.
            let Some(&start_index) = self.ids_to_indices.get(&root) else {
                continue;
            };
            // Use DFS to determine all reachable nodes from the root
            let dfs = Dfs::new(&self.graph, start_index);
            reachable.extend(dfs.iter(&self.graph).map(|index| self.indices_to_ids[&index]));
        }

        reachable
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

/// Compute for each function the set of functions called by it, and how many times it does so.
fn compute_callees(ssa: &Ssa) -> BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> {
    ssa.functions
        .iter()
        .flat_map(|(caller_id, function)| {
            let called_functions = called_functions_vec(function);
            called_functions.into_iter().map(|callee_id| (*caller_id, callee_id))
        })
        .fold(
            // Make sure an entry exists even for ones that don't call anything.
            ssa.functions.keys().map(|id| (*id, BTreeMap::new())).collect(),
            |mut acc, (caller_id, callee_id)| {
                let callees = acc.entry(caller_id).or_default();
                *callees.entry(callee_id).or_default() += 1;
                acc
            },
        )
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        ir::{call_graph::CallGraph, map::Id},
        ssa_gen::Ssa,
    };

    #[test]
    fn mark_mutually_recursive_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        brillig(inline) fn starter f1 {
          b0():
            call f2()
            return
        }
        brillig(inline) fn ping f2 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn pong f3 {
          b0():
            call f2()
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let recursive_functions = call_graph.get_recursive_functions();

        assert_eq!(recursive_functions.len(), 2);
        assert!(recursive_functions.contains(&Id::test_new(2)));
        assert!(recursive_functions.contains(&Id::test_new(3)));
    }

    #[test]
    fn mark_multiple_independent_recursion_cycles() {
        // This test is an expanded version of `mark_mutually_recursive_functions` where we have multiple recursive cycles.
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            call f4()
            return
        }
        // First recursive cycle: f1 -> f2 -> f3 -> f1
        brillig(inline) fn starter f1 {
          b0():
            call f2()
            return
        }
        brillig(inline) fn ping f2 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn pong f3 {
          b0():
            call f1()
            return
        }
        // Second recursive cycle: f4 <-> f5
        brillig(inline) fn foo f4 {
          b0():
            call f5()
            return
        }
        brillig(inline) fn bar f5 {
          b0():
            call f4()
            return
        }
        // Non-recursive leaf function
        brillig(inline) fn baz f6 {
          b0(): 
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let recursive_functions = call_graph.get_recursive_functions();

        // There should be 5 recursive functions: f1, f2, f3 (cycle 1), and f4, f5 (cycle 2)
        let expected_recursive_ids = [1, 2, 3, 4, 5].map(Id::test_new).to_vec();

        assert_eq!(
            recursive_functions.len(),
            expected_recursive_ids.len(),
            "Expected {} recursive functions",
            expected_recursive_ids.len()
        );

        for func_id in expected_recursive_ids {
            assert!(
                recursive_functions.contains(&func_id),
                "Function {func_id} should be marked recursive",
            );
        }

        // f6 should not be marked recursive
        assert!(!recursive_functions.contains(&Id::test_new(6)));
    }

    #[test]
    fn mark_self_recursive_function() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        brillig(inline) fn self_recur f1 {
          b0():
            call f1()
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let recursive_functions = call_graph.get_recursive_functions();

        assert_eq!(recursive_functions.len(), 1);
        assert!(recursive_functions.contains(&Id::test_new(1)));
    }

    #[test]
    fn self_recursive_and_calls_others() {
        let src = "
        acir(inline) fn main f0 {
          b0(): 
            call f1()
            return
        }
        brillig(inline) fn self_recur f1 {
          b0():
            call f1()
            call f2()
            return
        }
        brillig(inline) fn foo f2 {
          b0(): 
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);

        let f0 = Id::test_new(0);
        let f1 = Id::test_new(1);
        let f2 = Id::test_new(2);

        let recursive = call_graph.get_recursive_functions();
        assert!(recursive.contains(&f1));
        assert!(!recursive.contains(&f0));
        assert!(!recursive.contains(&f2));

        let callees = call_graph.callees();
        let f1_callees = callees.get(&f1).unwrap();
        assert_eq!(f1_callees.len(), 2);
        assert_eq!(*f1_callees.get(&f1).unwrap(), 1, "f1 should call itself once");
        assert_eq!(*f1_callees.get(&f2).unwrap(), 1, "f1 should call f2 once");

        let callers = call_graph.callers();
        let f1_callers = callers.get(&f1).unwrap();
        assert_eq!(f1_callers.len(), 2);
        assert_eq!(*f1_callers.get(&f0).unwrap(), 1, "f0 calls f1 once");
        assert_eq!(*f1_callers.get(&f1).unwrap(), 1, "f1 calls itself once");

        let f2_callers = callers.get(&f2).unwrap();
        assert_eq!(f2_callers.len(), 1);
        assert_eq!(*f2_callers.get(&f1).unwrap(), 1, "f1 calls f2 once");

        let f2_callees = callees.get(&f2).unwrap();
        assert!(f2_callees.is_empty(), "f2 should not call any functions");

        let f0_callees = callees.get(&f0).unwrap();
        assert_eq!(f0_callees.len(), 1);
        assert_eq!(*f0_callees.get(&f1).unwrap(), 1);
    }

    #[test]
    fn pure_self_recursive_function() {
        let src = "
        brillig(inline) fn self_recur f0 {
          b0(): 
            call f0()
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);

        let recursive = call_graph.get_recursive_functions();
        assert!(recursive.contains(&Id::test_new(0)));

        let callers = call_graph.callers();
        let f0_callers = callers.get(&Id::test_new(0)).unwrap();
        assert_eq!(f0_callers.len(), 1);
        assert_eq!(*f0_callers.get(&Id::test_new(0)).unwrap(), 1);
    }

    fn callers_and_callees_src() -> &'static str {
        r#"
        acir(inline) fn main f0 {
          b0():
            call f1()
            call f1()
            call f2()
            return
        }
        acir(inline) fn foo f1 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn bar f2 {
          b0():
            call f3()
            call f4()
            call f4()
            return
        }
        brillig(inline) fn baz f3 {
          b0():
            return
        }
        brillig(inline) fn qux f4 {
          b0():
            call f3()
            return
        }
        "#
    }

    #[test]
    fn callers() {
        let ssa = Ssa::from_str(callers_and_callees_src()).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let callers = call_graph.callers();

        let f0_callers = callers.get(&Id::test_new(0)).expect("Should have callers");
        assert!(f0_callers.is_empty());

        let f1_callers = callers.get(&Id::test_new(1)).expect("Should have callers");
        assert_eq!(f1_callers.len(), 1);
        let times_f1_called_by_f0 =
            *f1_callers.get(&Id::test_new(0)).expect("Should have times called");
        assert_eq!(times_f1_called_by_f0, 2);

        let f2_callers = callers.get(&Id::test_new(2)).expect("Should have callers");
        assert_eq!(f2_callers.len(), 1);
        let times_f2_called_by_f0 =
            *f2_callers.get(&Id::test_new(0)).expect("Should have times called");
        assert_eq!(times_f2_called_by_f0, 1);

        let f3_callers = callers.get(&Id::test_new(3)).expect("Should have callers");
        assert_eq!(f3_callers.len(), 3);
        let times_f3_called_by_f1 =
            *f3_callers.get(&Id::test_new(1)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f1, 1);
        let times_f3_called_by_f2 =
            *f3_callers.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f2, 1);
        let times_f3_called_by_f4 =
            *f3_callers.get(&Id::test_new(4)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f4, 1);

        let f4_callers = callers.get(&Id::test_new(4)).expect("Should have callers");
        assert_eq!(f4_callers.len(), 1);
        let times_f4_called_by_f2 =
            *f4_callers.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f4_called_by_f2, 2);
    }

    #[test]
    fn callees() {
        let ssa = Ssa::from_str(callers_and_callees_src()).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let callees = call_graph.callees();

        let f0_callees = callees.get(&Id::test_new(0)).expect("Should have callees");
        assert_eq!(f0_callees.len(), 2);
        let times_f0_calls_f1 =
            *f0_callees.get(&Id::test_new(1)).expect("Should have times called");
        assert_eq!(times_f0_calls_f1, 2);
        let times_f0_calls_f2 =
            *f0_callees.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f0_calls_f2, 1);

        let f1_callees = callees.get(&Id::test_new(1)).expect("Should have callees");
        assert_eq!(f1_callees.len(), 1);
        let times_f1_calls_f3 =
            *f1_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f1_calls_f3, 1);

        let f2_callees = callees.get(&Id::test_new(2)).expect("Should have callees");
        assert_eq!(f2_callees.len(), 2);
        let times_f2_calls_f3 =
            *f2_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f2_calls_f3, 1);
        let times_f2_calls_f4 =
            *f2_callees.get(&Id::test_new(4)).expect("Should have times called");
        assert_eq!(times_f2_calls_f4, 2);

        let f3_callees = callees.get(&Id::test_new(3)).expect("Should have callees");
        assert!(f3_callees.is_empty());

        let f4_callees = callees.get(&Id::test_new(4)).expect("Should have callees");
        let times_f4_calls_f3 =
            *f4_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f4_calls_f3, 1);
    }

    #[test]
    fn times_called() {
        let ssa = Ssa::from_str(callers_and_callees_src()).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let times_called = call_graph.times_called();

        let times_f0_called =
            *times_called.get(&Id::test_new(0)).expect(" Should have times called");
        assert_eq!(times_f0_called, 0);

        let times_f1_called =
            *times_called.get(&Id::test_new(1)).expect(" Should have times called");
        assert_eq!(times_f1_called, 2);

        let times_f2_called =
            *times_called.get(&Id::test_new(2)).expect(" Should have times called");
        assert_eq!(times_f2_called, 1);

        let times_f3_called =
            *times_called.get(&Id::test_new(3)).expect(" Should have times called");
        assert_eq!(times_f3_called, 3);

        let times_f4_called =
            *times_called.get(&Id::test_new(4)).expect(" Should have times called");
        assert_eq!(times_f4_called, 2);
    }

    #[test]
    fn dead_function_not_called() {
        let src = "
        acir(inline) fn main f0 {
          b0(): 
            return
        }
        brillig(inline) fn dead_code f1 {
          b0(): 
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);

        // f1 is never called, but it should still be tracked.
        let times_called = call_graph.times_called();
        assert_eq!(*times_called.get(&Id::test_new(1)).unwrap(), 0);
        assert!(call_graph.callers().get(&Id::test_new(1)).unwrap().is_empty());
        assert!(call_graph.callees().get(&Id::test_new(1)).unwrap().is_empty());
    }
}
