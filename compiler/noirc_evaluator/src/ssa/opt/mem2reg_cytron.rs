//! Mem2reg algorithm adapted from the Cytron paper: https://bernsteinbear.com/assets/img/cytron-ssa.pdf
use std::collections::BTreeSet;
use std::collections::hash_map::Entry;

use rustc_hash::FxHashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Instruction, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn mem2reg_cytron(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.mem2reg_simple();
        }
        self
    }
}

impl Function {
    fn mem2reg_simple(&mut self) {
        let mut has_already = FxHashMap::default();
        let mut work = FxHashMap::default();
        let mut w = BTreeSet::default();
        let mut iter_count = 0;

        let variables = collect_variables(self);
        let cfg = ControlFlowGraph::with_function(self);
        let dominance_frontiers =
            DominatorTree::with_function(self).compute_dominance_frontiers(&cfg);

        // Iterate through all variables
        for v in variables.keys() {
            iter_count += 1;

            // Iterate through all blocks where `v` is stored
            for x in variables[v].iter() {
                work.insert(*x, iter_count);
                w.insert(*x);
            }

            while let Some(x) = w.pop_first() {
                if let Some(frontier) = dominance_frontiers.get(&x) {
                    for y in frontier {
                        if get_or_zero(&has_already, y) < iter_count {
                            add_block_param_and_args(self, &cfg, *v, *y);
                            has_already.insert(*y, iter_count);

                            if get_or_zero(&work, y) < iter_count {
                                work.insert(*y, iter_count);
                                w.insert(*y);
                            }
                        }
                    }
                }
            }
        }
    }

    fn search_init(&self, variables: &Variables) {
        let mut c = FxHashMap::default();
        let mut s = FxHashMap::default();

        for each variable V {
            c(v) <- o
            S(V) <- []
        }
        self.search(self.entry_block());
    }

    fn search(&self, X: BasicBlockId, cfg: &ControlFlowGraph, C: &mut FxHashMap<ValueId, u32>, S: &mut FxHashMap<ValueId, Vec<u32>>) {
        for A in self.dfg[X].instructions() {
            if let Instruction::Store { address: _, value } = self.dfg[A] {
                for each variable V used In value {
                    replace use of V by use of Vi, where i = Top(S(V))
                }
            }

            for V in LHS(A) {
                let i = C[V];
                replace V by new Vi in LHS(A)
                S.get_mut(V).unwrap().push(i);
                C.get_mut(V).unwrap() += 1;
            }
        }

        for Y in cfg.successors(X) {
            let j = WhichPred(cfg, Y, X);
            for phiFunction F in Y {
                replace the j-th operand V in RHS(F) by Vi, where i = Top(S(V))
            }
        }

        for each Y in Children(X) {
            call SEARCH(Y)
        }

        for each assignment A in X {
            for each V in oldLHS(A) {
                pop s(v)
            }
        }
    }
}

fn WhichPred(cfg: &ControlFlowGraph, Y: BasicBlockId, X: BasicBlockId) -> usize {
    cfg.predecessors(Y)
        .position(|pred| pred == X)
        .expect("X is not a predecessor of Y")
}

/// Helper to get a value from a map, returning zero if the key is not present
fn get_or_zero(map: &FxHashMap<BasicBlockId, usize>, key: &BasicBlockId) -> usize {
    map.get(key).copied().unwrap_or(0)
}

fn add_block_param_and_args(
    function: &mut Function,
    cfg: &ControlFlowGraph,
    var: ValueId,
    block: BasicBlockId,
) {
    let parameter_type = function.dfg.type_of_value(var);
    let param = function.dfg.add_block_parameter(block, parameter_type);

    for predecessor in cfg.predecessors(block) {
        match function.dfg[predecessor].unwrap_terminator_mut() {
            TerminatorInstruction::Jmp { arguments, .. } => arguments.push(param),
            other => panic!("Unexpected terminator when adding block argument: {other:?}"),
        }
    }
}

/// Each variable in the program along with the blocks they're stored in, excluding the initial store.
///
/// A variable here is always the result of a `allocate` instruction.
type Variables = FxHashMap<ValueId, BTreeSet<BasicBlockId>>;

/// Collect all variables in the function along with the blocks they are stored in
fn collect_variables(function: &Function) -> Variables {
    let mut variables = Variables::default();

    // Block iteration order does not matter here
    for block in function.reachable_blocks() {
        for instruction_id in function.dfg[block].instructions() {
            let instruction = &function.dfg[*instruction_id];
            match instruction {
                Instruction::Allocate => {
                    let result = function.dfg.instruction_results(*instruction_id)[0];
                    new_variable(&mut variables, result);
                }
                _ => instruction.for_each_value(|value| use_value(&mut variables, value, block)),
            }
        }
    }
    variables
}

/// If the given value is a known variable, remember its use in the current block
fn use_value(variables: &mut Variables, value: ValueId, block: BasicBlockId) {
    if let Entry::Occupied(mut entry) = variables.entry(value) {
        entry.get_mut().insert(block);
    }
}

/// Records a new variable in the set of known variables, with no blocks used
fn new_variable(variables: &mut Variables, value: ValueId) {
    variables.entry(value).or_default();
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::mem2reg_cytron::collect_variables, ssa_gen::Ssa},
    };

    #[test]
    fn test_simple() {
        let src = "
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v3 = make_array [Field 1, Field 1] : [Field; 2]
            store v3 at v0
            v4 = load v0 -> [Field; 2]
            v5 = array_get v4, index u32 1 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_cytron();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v2 = make_array [Field 1, Field 1] : [Field; 2]
            return Field 1
        }
        ");
    }

    #[test]
    fn test_multiblock() {
        let src = "
        acir(inline) fn func f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmpif v0 then: b1, else: b2
          b1():
            store Field 1 at v1
            jmp b3()
          b2():
            store Field 2 at v1
            jmp b3()
          b3():
            v4 = load v1 -> Field
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_cytron();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut Field
            return Field 1
        }
        ");
    }
}
