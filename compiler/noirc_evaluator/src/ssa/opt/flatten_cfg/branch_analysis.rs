//! This is an algorithm for identifying branch starts and ends.
//!
//! The algorithm is split into two parts:
//! 1. The outer part:
//!    1. An (unrolled) CFG can be though of as a linear sequence of blocks where some nodes split
//!       off, but eventually rejoin to a new node and continue the linear sequence.
//!
//!    2. Follow this sequence in order, and whenever a split is found call
//!       `find_join_point_of_branches` and then recur from the join point it returns until the
//!       return instruction is found.
//!
//! 2. The inner part defined by `find_join_point_of_branches`:
//!
//!    The recursive version of the algorithm is as follows:
//!    1. For each of the two branches in a jmpif block:
//!     - Check if either has multiple predecessors. If so, it is a join point.
//!     - If not, continue to search the linear sequence of successor blocks from that block.
//!       - If another split point is found, recur in `find_join_point_of_branches`
//!       - If a block with multiple predecessors is found, return it.
//!     - After, we should have identified a join point for both branches. This is expected to be
//!       the same block for both and can be returned from here to continue iteration.
//!
//!    The recursive variant can encounter a stack overflow on certain CFG.
//!
//!    The iterative variant goes like this:
//!    1. Pop the next branch from a stack of branches we need to process.
//!     - Find the next join or branch point on both left and right branches.
//!     - If the branches rejoin immediately:
//!       - Repeatedly find the next point following the join point:
//!         - If it's another join point, it must be for the parent level:
//!           - If this is the first time we see a join point for this level, mark it as pending.
//!           - If it's the second time, mark it as the end point, ensuring it matches the pending value.
//!         - If a new branch is encountered, push it onto the stack, noting the parent level to return to.
//!     - If either branches are followed by further branching:
//!       - Push the branching children onto the stack for visiting later, noting to return to the current level.
//!       - Mark any children that goes to in a join point as a pending end for this branch.
//!
//! This algorithm will remember each join point found in `find_join_point_of_branches` and
//! the resulting map from each split block to each join block is returned.

use std::collections::HashSet;

use crate::ssa::ir::{basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function};
use rustc_hash::FxHashMap as HashMap;

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

/// The next interesting point following some block.
#[derive(Debug, Clone)]
enum Point {
    /// The CFG rejoined.
    Join(BasicBlockId),
    /// The CFG branched again at a parent block into a left and right child.
    Branch(BasicBlockId, BasicBlockId, BasicBlockId),
}

struct Context<'cfg> {
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,
    branch_ends_pending: HashMap<BasicBlockId, BasicBlockId>,
    branch_parents: HashMap<BasicBlockId, BasicBlockId>,
    stack: Vec<(BasicBlockId, BasicBlockId, BasicBlockId)>,
    cfg: &'cfg ControlFlowGraph,
}

impl<'cfg> Context<'cfg> {
    fn new(cfg: &'cfg ControlFlowGraph) -> Self {
        Self {
            cfg,
            branch_ends: HashMap::default(),
            branch_ends_pending: HashMap::default(),
            branch_parents: HashMap::default(),
            stack: Vec::new(),
        }
    }

    fn find_join_point_of_branches(
        &mut self,
        start: BasicBlockId,
        mut successors: impl Iterator<Item = BasicBlockId>,
    ) -> BasicBlockId {
        let left = successors.next().unwrap();
        let right = successors.next().unwrap();
        let mut visited = HashSet::new();

        // Kick off the stack from the starting branch. It doesn't have a parent.
        self.stack.push((start, left, right));

        while let Some((branch, left, right)) = self.stack.pop() {
            if !visited.insert(branch) {
                continue;
            }
            let left = self.find_next_point(left, false);
            let right = self.find_next_point(right, false);

            if let Some(join) = self.maybe_join(branch, &left, &right) {
                // If we managed to join the branches immediately, then we know where this branch ends,
                // and we can check if we can complete any parent levels.
                self.complete_parents(branch, join);
            } else {
                // At least one of the children further branches off.
                for child in [left, right] {
                    match child {
                        Point::Join(next) => {
                            self.branch_ends_pending.insert(branch, next);
                        }
                        Point::Branch(next, left, right) => {
                            self.push_branch(branch, next, left, right);
                        }
                    }
                }
            }
        }

        self.branch_ends
            .get(&start)
            .cloned()
            .unwrap_or_else(|| panic!("should have found the join point for {start}"))
    }

    /// Try to complete as many of the parent levels as we can, after finding the end point of a branch.
    ///
    /// Say we have this CFG:
    /// ```text
    ///                    b11
    ///                   /   \
    ///       b5        b9     b13
    ///      /  \      /  \   /   \
    ///    b1    b7--b8    b12     b15
    ///    / \  /      \          /   \
    ///  b0   b6        b10----b14    b4
    ///    \                          /
    ///    b2-----------------------b3
    /// ```
    /// At some point during the algorithm:
    /// * We process `b8` we see that on the `b10` side it's followed by the join point `b15` and on the `b9` side it's another branch.
    /// * We take note of `b15` as the pending ending for `b8`, and queue `b9` for processing.
    /// * When we process `b9`, we get the join point `b13` on both `b11` and `b12`, so we "immediately" know that `b9` ends at `b13`.
    /// * At that point we look for the next point after `b13`, which is the join point `b15`.
    /// * The parent of `b9` is `b8`, which already has `b15` as a pending end, which checks out, so we can "complete" `b8`.
    /// * Then we follow up the chain: the parent level of `b8` is `b0`; `b15` is followed by `b4`; `b0` already has `b4` as pending end, so we can "complete" that as well.
    fn complete_parents(&mut self, mut branch: BasicBlockId, mut join: BasicBlockId) {
        loop {
            // If we reached the starting point, we can stop.
            let Some(parent) = self.branch_parents.get(&branch).cloned() else {
                break;
            };
            // We can skip this join point (we know it completes this level, not the parent), and look for the next one.
            match self.find_next_point(join, true) {
                Point::Join(next) => {
                    // If it's a second join point, then we went back to the parent level. Try to complete it.
                    if self.maybe_join_pending(parent, next) {
                        branch = parent;
                        join = next;
                    } else {
                        break;
                    }
                }
                Point::Branch(next, left, right) => {
                    // We found another branch on the same level as we are currently at.
                    // We must visit it before we can return the the previous level.
                    self.push_branch(parent, next, left, right);
                    break;
                }
            }
        }
    }

    /// Push a branch to the stack for exploration, remembering what the parent level branch was.
    fn push_branch(
        &mut self,
        parent: BasicBlockId,
        branch: BasicBlockId,
        left: BasicBlockId,
        right: BasicBlockId,
    ) {
        self.stack.push((branch, left, right));
        // Remember where we need to go back to.
        self.branch_parents.insert(branch, parent);
    }

    /// Check if the left and right branches joined.
    /// If so, mark the branch as completed, and return `Some` join point, otherwise return `None`.
    /// Panics if both points are joins, but have different values.
    fn maybe_join(
        &mut self,
        branch: BasicBlockId,
        left: &Point,
        right: &Point,
    ) -> Option<BasicBlockId> {
        if let (Point::Join(left), Point::Join(right)) = (left, right) {
            self.must_join(branch, *left, *right);
            Some(*left)
        } else {
            None
        }
    }

    /// Try to join a pending branch, once the next join point is found:
    /// * if this is the first time we encounter this, mark it as pending, and return `false`,
    /// * if this is the second time, then we mark it as completed, and return `true`.
    ///
    /// Panics if the join point does not match the existing one.
    fn maybe_join_pending(&mut self, parent: BasicBlockId, join: BasicBlockId) -> bool {
        let Some(pending) = self.branch_ends_pending.insert(parent, join) else {
            return false;
        };
        self.must_join(parent, pending, join);
        true
    }

    /// Check that the left and right join points are the same, and mark this as the join point for the start block.
    fn must_join(&mut self, start: BasicBlockId, left: BasicBlockId, right: BasicBlockId) {
        assert_eq!(left, right, "Expected two blocks to join to the same block");
        self.branch_ends.insert(start, left);
    }

    /// Starting with the current block, find the next join or branching point,
    /// skipping over blocks with single successors and predecessors.
    ///
    /// If a block is both a join and a branch block, it is returned as a join.
    ///
    /// When `skip` is true, we don't consider the current block for an immediate join point,
    /// although it can be returned as a branch.
    fn find_next_point(&self, mut block: BasicBlockId, mut skip: bool) -> Point {
        loop {
            if !skip {
                let predecessors = self.cfg.predecessors(block);
                if predecessors.len() > 1 {
                    return Point::Join(block);
                }
            }
            let mut successors = self.cfg.successors(block);
            match successors.len() {
                2 => {
                    let left = successors.next().unwrap();
                    let right = successors.next().unwrap();
                    return Point::Branch(block, left, right);
                }
                1 => {
                    skip = false;
                    block = successors.next().unwrap();
                }
                0 => {
                    unreachable!(
                        "return encountered before a join point was found. This can only happen if early-return was added to the language without implementing it by jumping to a join block first"
                    );
                }
                _ => {
                    unreachable!("A block can only have 0, 1, or 2 successors");
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        brillig::BrilligOptions,
        ssa::{
            SsaEvaluatorOptions,
            function_builder::FunctionBuilder,
            ir::{basic_block::BasicBlockId, cfg::ControlFlowGraph, map::Id, types::Type},
            opt::{constant_folding, flatten_cfg::branch_analysis::find_branch_ends, inlining},
            primary_passes,
            ssa_gen::Ssa,
        },
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

        let ssa = builder.finish();
        let function = ssa.main();
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

        let ssa = builder.finish();
        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);
        find_branch_ends(function, &cfg);
    }

    #[test]
    fn test_find_branch_ends_with_documented_example() {
        //                    b11
        //                   /   \
        //       b5        b9     b13
        //      /  \      /  \   /   \
        //    b1    b7--b8    b12     b15
        //    / \  /      \          /   \
        //  b0   b6        b10----b14    b4
        //    \                          /
        //    b2-----------------------b3
        let src = "
        acir(inline) fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            jmpif u1 0 then: b5, else: b6
          b2():
            jmp b3()
          b3():
            jmp b4()
          b4():
            return
          b5():
            jmp b7()
          b6():
            jmp b7()
          b7():
            jmp b8()
          b8():
            jmpif u1 0 then: b9, else: b10
          b9():
            jmpif u1 0 then: b11, else: b12
          b10():
            jmp b14()
          b11():
            jmp b13()
          b12():
            jmp b13()
          b13():
            jmp b15()
          b14():
            jmp b15()
          b15():
            jmp b4()
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();
        let cfg = ControlFlowGraph::with_function(main);
        let ends = find_branch_ends(main, &cfg);

        let b = |n| BasicBlockId::new(n);
        assert_eq!(ends[&b(0)], b(4));
        assert_eq!(ends[&b(1)], b(7));
        assert_eq!(ends[&b(8)], b(15));
        assert_eq!(ends[&b(9)], b(13));
    }

    #[test]
    fn apply_function() {
        // Make sure that our dynamic dispatch function created during defunctionalization
        // passes branch analysis.
        let src = "
        acir(inline_always) fn apply f5 {
          b0(v0: Field, v1: u32):
            v4 = eq v0, Field 2
            jmpif v4 then: b3, else: b2
          b1(v2: u32):
            return v2
          b2():
            v9 = eq v0, Field 3
            jmpif v9 then: b6, else: b5
          b3():
            v6 = call f2(v1) -> u32
            jmp b4(v6)
          b4(v7: u32):
            jmp b10(v7)
          b5():
            constrain v0 == Field 4
            v15 = call f4(v1) -> u32
            jmp b8(v15)
          b6():
            v11 = call f3(v1) -> u32
            jmp b7(v11)
          b7(v12: u32):
            jmp b9(v12)
          b8(v16: u32):
            jmp b9(v16)
          b9(v17: u32):
            jmp b10(v17)
          b10(v18: u32):
            jmp b1(v18)
        }
        acir(inline) fn lambda f2 {
          b0(v0: u32):
            return v0
        }
        acir(inline) fn lambda f3 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        acir(inline) fn lambda f4 {
          b0(v0: u32):
            v2 = add v0, u32 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);
        find_branch_ends(function, &cfg);
    }

    #[test]
    fn test_large_unroll_stack_overflow() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u64, v2: i32, v3: u1, v4: Field, v5: u1, v6: u1):
            jmpif v6 then: b1, else: b2
          b1():
            jmp b14()
          b2():
            jmp b5(u32 67)
          b3():
            jmp b8(u32 198)
          b4():
            jmp b14()
          b5(v7: u32):
            v11 = lt v7, u32 232
            jmpif v11 then: b3, else: b4
          b6():
            v18 = add v7, u32 1
            v19 = truncate v18 to 32 bits, max_bit_size: 33
            v21 = call f1(v4, v4, v4, v4, v4, v0, v0) -> Field
            jmp b5(v19)
          b7():
            jmpif v6 then: b10, else: b11
          b8(v8: u32):
            v14 = lt v8, u32 90
            jmpif v14 then: b7, else: b6
          b9():
            v16 = add v8, u32 1
            v17 = truncate v16 to 32 bits, max_bit_size: 33
            jmp b8(v17)
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            jmp b9()
          b13():
            return v0
          b14():
            jmp b13()
        }
        acir(inline) fn f1 f1 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
            jmp b3(u32 54)
          b1():
            jmpif v6 then: b5, else: b6
          b2():
            jmp b8()
          b3(v7: u32):
            v10 = lt v7, u32 207
            jmpif v10 then: b1, else: b2
          b4():
            v12 = add v7, u32 1
            v13 = truncate v12 to 32 bits, max_bit_size: 33
            jmp b3(v13)
          b5():
            jmp b7()
          b6():
            jmp b7()
          b7():
            jmp b4()
          b8():
            return v4
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();

        // If we try to run the branch analysis now, it panics, it doesn't have the expected CFG structure.
        // Instead, run the pipeline up to just before the flattening pass.
        let ssa = run_pipeline_up_to_pass(ssa, "Flattening");

        // The resulting SSA has more than 70k blocks.
        // Both functions in the SSA have blocks such as b1->[b5, b6]->b7 where
        // a branch immediately rejoins. For whatever combination, it's too much for
        // a recursive algorithm, and can cause a stack overflow.

        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);
        let _ = find_branch_ends(function, &cfg);
    }

    fn run_pipeline_up_to_pass(mut ssa: Ssa, stop_before_pass: &str) -> Ssa {
        let options = SsaEvaluatorOptions {
            ssa_logging: crate::ssa::SsaLogging::None,
            brillig_options: BrilligOptions::default(),
            print_codegen_timings: false,
            emit_ssa: None,
            skip_underconstrained_check: true,
            skip_brillig_constraints_check: true,
            enable_brillig_constraints_check_lookback: false,
            inliner_aggressiveness: 0,
            constant_folding_max_iter: constant_folding::DEFAULT_MAX_ITER,
            small_function_max_instruction: inlining::MAX_INSTRUCTIONS,
            max_bytecode_increase_percent: None,
            skip_passes: Vec::new(),
        };
        let pipeline = primary_passes(&options);
        for pass in pipeline {
            if pass.msg() == stop_before_pass {
                break;
            }
            ssa = pass
                .run(ssa)
                .unwrap_or_else(|e| panic!("failed to run pass '{}': {e}", pass.msg()));
        }
        ssa
    }
}
