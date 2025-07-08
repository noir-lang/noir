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
//!     - If this is the first time we see the branch:
//!       - Find the next join point or branch point on both left and right branches of a jmpif block.
//!       - If both branches rejoin immediately:
//!         - Repeatedly find the next join point after them, until we are back on the branching level we started from.
//!         - If a new branch is encountered, push it onto the stack.
//!       - If either branches are followed by further branching:
//!         - Push the current back back on the stack for a second visit later.
//!         - Push the branching children onto the stack for the first visit.
//!     - If this is the second time we visit the branch:
//!       - By now we expect to have already visited all descendant branches.
//!       - Find the join point of both left and right branches:
//!         - If the branch directly ends in a join point, return it.
//!         - If it ends in another branch:
//!           - Look up the end of that branch in the cache.
//!           - Skip the end block, and look for the next join point, where it rejoins its parent.
//!         - Repeat until we find two join points in a row, which should be the join point of the parent.
//!       - Store the end of the branch in the cache.
//!
//! This algorithm will remember each join point found in `find_join_point_of_branches` and
//! the resulting map from each split block to each join block is returned.

use std::collections::HashSet;

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

/// The next interesting point following some block.
#[derive(Debug, Clone)]
enum Point {
    /// The CFG rejoined.
    Join(BasicBlockId),
    /// The CFG branched again at a parent block into a left and right child.
    Branch(BasicBlockId, BasicBlockId, BasicBlockId),
}

/// We need to visit branches twice:
/// * First we look at the next interesting point and decide if we need to recurse.
/// * Second time we should have visited both left and right branch already.
#[derive(Debug)]
enum Visit {
    First(BasicBlockId, BasicBlockId, BasicBlockId, usize),
    Second(BasicBlockId, Point, Point),
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
        let mut stack = Vec::new();
        let mut visited = HashSet::new();
        stack.push(Visit::First(start, left, right, 0));

        'processing: while let Some(visit) = stack.pop() {
            match visit {
                Visit::First(start, left, right, mut level) => {
                    if !visited.insert(start) {
                        continue;
                    }
                    let left = self.find_next_point(left, false);
                    let right = self.find_next_point(right, false);

                    if let Some(mut join) = self.maybe_join(start, &left, &right) {
                        // If we managed to join the branches immediately, we still might have to recurse,
                        // until we have done enough joins to get back at the original level, or we have
                        // encountered another branch before we could rejoin the original.
                        while level > 0 {
                            // Skip this join point: we know it is not for the parent block we originally branched off from.
                            match self.find_next_point(join, true) {
                                Point::Join(next) => {
                                    level -= 1;
                                    join = next;
                                }
                                Point::Branch(next, left, right) => {
                                    // We found another branch on the same level as we are currently at.
                                    // We must visit it before we can return the the previous level.
                                    // Since this could be after a join point of multiple branches,
                                    // we could arrive here from two different ways.
                                    stack.push(Visit::First(next, left, right, level));
                                    continue 'processing;
                                }
                            }
                        }
                    } else {
                        // At least one of the branches further branches off, so we must schedule a second visit
                        // after we have processed them.
                        stack.push(Visit::Second(start, left.clone(), right.clone()));
                        for child in [left, right] {
                            if let Point::Branch(start, left, right) = child {
                                stack.push(Visit::First(start, left, right, level + 1));
                            }
                        }
                    }
                }
                Visit::Second(start, left, right) => {
                    let left = self.get_join_point(left);
                    let right = self.get_join_point(right);
                    self.must_join(start, left, right);
                }
            }
        }

        self.branch_ends
            .get(&start)
            .cloned()
            .unwrap_or_else(|| panic!("should have found the join point for {start}"))
    }

    /// Check if the left and right branches joined.
    /// If so, they are expected to have joined at the same block;
    /// remember that join point for the start and return the join point.
    /// If not, return `None`.
    fn maybe_join(
        &mut self,
        start: BasicBlockId,
        left: &Point,
        right: &Point,
    ) -> Option<BasicBlockId> {
        if let (Point::Join(left), Point::Join(right)) = (left, right) {
            self.must_join(start, *left, *right);
            Some(*left)
        } else {
            None
        }
    }

    /// Check that the left and right join points are the same, and mark this as the join point for the start block.
    fn must_join(&mut self, start: BasicBlockId, left: BasicBlockId, right: BasicBlockId) {
        assert_eq!(left, right, "Expected two blocks to join to the same block");
        self.branch_ends.insert(start, left);
    }

    /// Get the join point of a branch after all descendants have been visited,
    /// going from either the left or the right child, recursively getting the
    /// join points of branches, and then the last join point.
    fn get_join_point(&self, mut next: Point) -> BasicBlockId {
        loop {
            match next {
                Point::Join(id) => return id,
                Point::Branch(id, _, _) => {
                    // If we branched off, that means we have to repeatedly find join
                    // blocks after the branch rejoins, until we find one extra join
                    // block that brings us back to the current branching level.
                    let join = self
                        .branch_ends
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| panic!("branch end for {id} not found"));
                    // Skip this join point, it was for the child branch.
                    // If we branch off again (maybe from the same block),
                    // then do another pass over it, looking for the following
                    // join point, until we get two in a row.
                    next = self.find_next_point(join, true);
                }
            }
        }
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
                        "return encountered before a join point was found. This can only happen if early-return was added to the language without implementing it by jmping to a join block first"
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

    use acvm::acir::circuit::ExpressionWidth;

    use crate::{
        brillig::BrilligOptions,
        ssa::{
            SsaEvaluatorOptions,
            function_builder::FunctionBuilder,
            ir::{cfg::ControlFlowGraph, map::Id, types::Type},
            opt::flatten_cfg::branch_analysis::find_branch_ends,
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
            expression_width: ExpressionWidth::default(),
            emit_ssa: None,
            skip_underconstrained_check: true,
            skip_brillig_constraints_check: true,
            enable_brillig_constraints_check_lookback: false,
            inliner_aggressiveness: 0,
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
