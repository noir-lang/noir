//! This is an algorithm for identifying branch starts and ends.
//!
//! The algorithm is split into two parts:
//! 1. The outer part:
//!    A. An (unrolled) CFG can be though of as a linear sequence of blocks where some nodes split
//!       off, but eventually rejoin to a new node and continue the linear sequence.
//!    B. Follow this sequence in order, and whenever a split is found call
//!       `find_join_point_of_branches` and then recur from the join point it returns until the
//!       return instruction is found.
//!
//! 2. The inner part defined by `find_join_point_of_branches`:
//!    A. For each of the two branches in a jmpif block:
//!     - Check if either has multiple predecessors. If so, it is a join point.
//!     - If not, continue to search the linear sequence of successor blocks from that block.
//!       - If another split point is found, recur in `find_join_point_of_branches`
//!       - If a block with multiple predecessors is found, return it.
//!     - After, we should have identified a join point for both branches. This is expected to be
//!       the same block for both and can be returned from here to continue iteration.
//!
//! This algorithm will remember each join point found in `find_join_point_of_branches` and
//! the resulting map from each split block to each join block is returned.

use crate::ssa::ir::{basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function};
use fxhash::FxHashMap as HashMap;

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
    let mut block = function.entry_block();
    let mut context = Context::new(cfg);

    loop {
        let mut successors = cfg.successors(block);

        if successors.len() == 2 {
            block = context.find_join_point_of_branches(block, successors);
        } else if successors.len() == 1 {
            block = successors.next().unwrap();
        } else if successors.len() == 0 {
            // return encountered. We have nothing to join, so we're done
            break;
        } else {
            unreachable!("A block can only have 0, 1, or 2 successors");
        }
    }

    context.branch_ends
}

struct Context<'cfg> {
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,
    cfg: &'cfg ControlFlowGraph,
}

impl<'cfg> Context<'cfg> {
    fn new(cfg: &'cfg ControlFlowGraph) -> Self {
        Self { cfg, branch_ends: HashMap::default() }
    }

    fn find_join_point_of_branches(
        &mut self,
        start: BasicBlockId,
        mut successors: impl Iterator<Item = BasicBlockId>,
    ) -> BasicBlockId {
        let left = successors.next().unwrap();
        let right = successors.next().unwrap();

        let left_join = self.find_join_point(left);
        let right_join = self.find_join_point(right);

        assert_eq!(left_join, right_join, "Expected two blocks to join to the same block");
        self.branch_ends.insert(start, left_join);

        left_join
    }

    fn find_join_point(&mut self, block: BasicBlockId) -> BasicBlockId {
        let predecessors = self.cfg.predecessors(block);
        if predecessors.len() > 1 {
            return block;
        }
        // The join point is not this block, so continue on
        self.skip_then_find_join_point(block)
    }

    fn skip_then_find_join_point(&mut self, block: BasicBlockId) -> BasicBlockId {
        let mut successors = self.cfg.successors(block);

        if successors.len() == 2 {
            let join = self.find_join_point_of_branches(block, successors);
            // Note that we call skip_then_find_join_point here instead of find_join_point.
            // We already know this `join` is a join point, but it cannot be for the current block
            // since we already know it is the join point of the successors of the current block.
            self.skip_then_find_join_point(join)
        } else if successors.len() == 1 {
            self.find_join_point(successors.next().unwrap())
        } else if successors.len() == 0 {
            unreachable!("return encountered before a join point was found. This can only happen if early-return was added to the language without implementing it by jmping to a join block first")
        } else {
            unreachable!("A block can only have 0, 1, or 2 successors");
        }
    }
}

#[cfg(test)]
mod test {

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{cfg::ControlFlowGraph, map::Id, types::Type},
        opt::flatten_cfg::branch_analysis::find_branch_ends,
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
        let mut builder = FunctionBuilder::new("main".into(), main_id);

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
        let mut builder = FunctionBuilder::new("main".into(), main_id);

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
