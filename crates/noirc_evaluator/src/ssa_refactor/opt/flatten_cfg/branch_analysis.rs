//! This is an algorithm for identifying branch starts and ends.
use std::collections::{HashMap, HashSet};

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId, cfg::ControlFlowGraph, dom::DominatorTree, function::Function,
    post_order::PostOrder,
};

/// Returns a `HashMap` mapping blocks that start a branch (i.e. blocks terminated with jmpif) to
/// their corresponding blocks that end the branch.
///
/// This implementation assumes all branches re-converge. It would be little work to change it to
/// support non-convergence (i.e. for multiple returns), with the caveat that there would be some
/// inefficiency when processing such CFGs.
pub(super) fn find_branch_ends(
    function: &Function,
    cfg: &ControlFlowGraph,
) -> HashMap<BasicBlockId, BasicBlockId> {
    let post_order = PostOrder::with_function(function);
    let dom_tree = DominatorTree::with_cfg_and_post_order(cfg, &post_order);
    let mut stepper = Stepper::new(function.entry_block());
    // This outer `visited` set is inconsequential, and simply here to satisfy the recursive
    // stepper interface.
    let mut visited = HashSet::new();
    let mut branch_ends = HashMap::new();
    while !stepper.finished {
        stepper.step(cfg, &dom_tree, &mut visited, &mut branch_ends);
    }
    branch_ends
}

/// Returns the block at which `left` and `right` converge, at the same time identifying branch
/// ends in any sub branches.
///
/// This function is called by `Stepper::step` and is thus recursive.
fn step_until_rejoin(
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
    branch_ends: &mut HashMap<BasicBlockId, BasicBlockId>,
    left: BasicBlockId,
    right: BasicBlockId,
) -> BasicBlockId {
    let mut visited = HashSet::new();
    let mut left_stepper = Stepper::new(left);
    let mut right_stepper = Stepper::new(right);

    while !left_stepper.finished || !right_stepper.finished {
        left_stepper.step(cfg, dom_tree, &mut visited, branch_ends);
        right_stepper.step(cfg, dom_tree, &mut visited, branch_ends);
    }
    let collision = match (left_stepper.collision, right_stepper.collision) {
        (Some(collision), None) | (None, Some(collision)) => collision,
        (Some(_),Some(_))=> unreachable!("A collision on both branches indicates a loop"), 
        _ => unreachable!(
            "Until we support multiple returns, branches always re-converge. Once supported this case should return `None`"
        ),
    };
    collision
}

/// Tracks traversal along the arm of a branch. Steppers are progressed in pairs, such that the
/// re-convergence point of two arms is discovered as soon as possible. The exceptional case is
/// that of the top level stepper, which conveniently steps the whole CFG as if it were a single
/// arm.
struct Stepper {
    /// The block that will be interrogated when calling `step`
    current_block: BasicBlockId,
    /// Indicates that the stepper has no more block successors to process, either because it has
    /// reached the end of the CFG, or because it encountered a block already visited by its
    /// sibling stepper.
    finished: bool,
    /// Once finished this option indicates whether a collision was encountered before reaching
    /// the end of the CFG.
    collision: Option<BasicBlockId>,
}

impl Stepper {
    /// Creates a fresh stepper instance
    fn new(current_block: BasicBlockId) -> Self {
        Stepper { current_block, finished: false, collision: None }
    }

    /// Checks the current block to see if it has already been visited and if so marks it as a
    /// collision. If a sub-branch is encountered `step_until_rejoin` is called to start a pair
    /// of child steppers stepping along its arms.
    ///
    /// It is safe to call this even when the stepper has reached its end.
    fn step(
        &mut self,
        cfg: &ControlFlowGraph,
        dom_tree: &DominatorTree,
        visited: &mut HashSet<BasicBlockId>,
        branch_ends: &mut HashMap<BasicBlockId, BasicBlockId>,
    ) {
        if self.finished {
            // The caller still needs to progress the other stepper, while this one sits idle.
            return;
        }
        if visited.contains(&self.current_block) {
            // The other stepper has already visited this block - thus this block is the
            // re.-convergence point.
            self.collision = Some(self.current_block);
            self.finished = true;
        }
        visited.insert(self.current_block);

        let mut successors = cfg.successors(self.current_block);
        match successors.len() {
            0 => {
                // Reached the end of the CFG without a collision - this will happen in the other
                // stepper assuming the CFG contains no early returns.
                self.finished = true;
            }
            1 => {
                // This block doesn't describe any branch starts or ends - move on.
                self.current_block = successors.next().unwrap();
            }
            2 => {
                // Sub-branch start encountered - recurse to find the end of the sub branch
                let left = successors.next().unwrap();
                let right = successors.next().unwrap();
                let sub_branch_end = step_until_rejoin(cfg, dom_tree, branch_ends, left, right);
                for collision_predecessor in cfg.predecessors(sub_branch_end) {
                    assert!(dom_tree.dominates(self.current_block, collision_predecessor));
                }
                branch_ends.insert(self.current_block, sub_branch_end);

                // Resume stepping though the current arm fro where the sub-branch left off
                self.current_block = sub_branch_end;
            }
            _ => {
                unreachable!("Basic blocks never have more than 2 successors")
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::ssa_refactor::{
        ir::{cfg::ControlFlowGraph, function::RuntimeType, map::Id, types::Type},
        opt::flatten_cfg::branch_analysis::find_branch_ends,
        ssa_builder::FunctionBuilder,
    };

    #[test]
    fn nested_branch_analysis() {
        //         b0
        //         ↓
        //         b1
        //       ↙   ↘
        //     b2     b3
        //     ↓      |
        //     b4     |
        //   ↙  ↘     |
        // b5    b6   |
        //   ↘  ↙     ↓
        //    b7      b8
        //      ↘   ↙
        //       b9
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        let b6 = builder.insert_block();
        let b7 = builder.insert_block();
        let b8 = builder.insert_block();
        let b9 = builder.insert_block();

        let c1 = builder.add_parameter(Type::bool());
        let c4 = builder.add_parameter(Type::bool());

        builder.terminate_with_jmp(b1, vec![]);
        builder.switch_to_block(b1);
        builder.terminate_with_jmpif(c1, b2, b3);
        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b4, vec![]);
        builder.switch_to_block(b3);
        builder.terminate_with_jmp(b8, vec![]);
        builder.switch_to_block(b4);
        builder.terminate_with_jmpif(c4, b5, b6);
        builder.switch_to_block(b5);
        builder.terminate_with_jmp(b7, vec![]);
        builder.switch_to_block(b6);
        builder.terminate_with_jmp(b7, vec![]);
        builder.switch_to_block(b7);
        builder.terminate_with_jmp(b9, vec![]);
        builder.switch_to_block(b8);
        builder.terminate_with_jmp(b9, vec![]);
        builder.switch_to_block(b9);
        builder.terminate_with_return(vec![]);

        let mut ssa = builder.finish();
        let function = ssa.main_mut();
        let cfg = ControlFlowGraph::with_function(function);
        let branch_ends = find_branch_ends(function, &cfg);
        assert_eq!(branch_ends.len(), 2);
        assert_eq!(branch_ends.get(&b1), Some(&b9));
        assert_eq!(branch_ends.get(&b4), Some(&b7));
    }

    #[test]
    fn more_nested_branch_analysis() {
        // Taken from #1664. The success case is that the internal domination asserts all pass.
        //          b0
        //        ↙   ↘
        //      b1     b10
        //    ↙  ↓      ↓  ↘
        // b2 → b3     b12 ← b11
        //    ↙  ↓      ↓  ↘
        // b4 → b5     b14 ← b13
        //    ↙  ↓      |
        // b6 → b7      |
        //    ↙  ↓      |
        // b8 → b9      |
        //        ↘    ↙
        //          b15
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        let b6 = builder.insert_block();
        let b7 = builder.insert_block();
        let b8 = builder.insert_block();
        let b9 = builder.insert_block();
        let b10 = builder.insert_block();
        let b11 = builder.insert_block();
        let b12 = builder.insert_block();
        let b13 = builder.insert_block();
        let b14 = builder.insert_block();
        let b15 = builder.insert_block();

        let c0 = builder.add_parameter(Type::bool());
        let c1 = builder.add_parameter(Type::bool());
        let c3 = builder.add_parameter(Type::bool());
        let c5 = builder.add_parameter(Type::bool());
        let c7 = builder.add_parameter(Type::bool());
        let c10 = builder.add_parameter(Type::bool());
        let c12 = builder.add_parameter(Type::bool());

        builder.terminate_with_jmpif(c0, b1, b10);
        builder.switch_to_block(b1);
        builder.terminate_with_jmpif(c1, b2, b3);
        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b3, vec![]);
        builder.switch_to_block(b3);
        builder.terminate_with_jmpif(c3, b4, b5);
        builder.switch_to_block(b4);
        builder.terminate_with_jmp(b5, vec![]);
        builder.switch_to_block(b5);
        builder.terminate_with_jmpif(c5, b6, b7);
        builder.switch_to_block(b6);
        builder.terminate_with_jmp(b7, vec![]);
        builder.switch_to_block(b7);
        builder.terminate_with_jmpif(c7, b8, b9);
        builder.switch_to_block(b8);
        builder.terminate_with_jmp(b9, vec![]);
        builder.switch_to_block(b9);
        builder.terminate_with_jmp(b15, vec![]);
        builder.switch_to_block(b10);
        builder.terminate_with_jmpif(c10, b11, b12);
        builder.switch_to_block(b11);
        builder.terminate_with_jmp(b12, vec![]);
        builder.switch_to_block(b12);
        builder.terminate_with_jmpif(c12, b14, b13);
        builder.switch_to_block(b13);
        builder.terminate_with_jmp(b14, vec![]);
        builder.switch_to_block(b14);
        builder.terminate_with_jmp(b15, vec![]);
        builder.switch_to_block(b15);
        builder.terminate_with_return(vec![]);

        let mut ssa = builder.finish();
        let function = ssa.main_mut();
        let cfg = ControlFlowGraph::with_function(function);
        find_branch_ends(function, &cfg);
    }
}
