use std::collections::{HashMap, HashSet};

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId, cfg::ControlFlowGraph, dom::DominatorTree, function::Function,
    post_order::PostOrder,
};

struct Context<'cfg> {
    cfg: &'cfg ControlFlowGraph,
    dom_tree: DominatorTree,
    // visited: HashSet<BasicBlockId>,
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,
}

pub(super) fn analyze_branch_ends(
    cfg: &ControlFlowGraph,
    function: &Function,
) -> HashMap<BasicBlockId, BasicBlockId> {
    let entry_block = function.entry_block();
    let mut context = Context::new(cfg, function);
    let mut visited_by_children = HashSet::new();
    context.analyze(entry_block, &mut visited_by_children);
    context.branch_ends
}

impl<'cfg> Context<'cfg> {
    fn new(cfg: &'cfg ControlFlowGraph, function: &Function) -> Self {
        let post_order = PostOrder::with_function(function);
        let dom_tree = DominatorTree::with_cfg_and_post_order(cfg, &post_order);
        Context { cfg, dom_tree, branch_ends: HashMap::new() }
    }

    fn analyze(
        &mut self,
        mut active_block: BasicBlockId,
        visited: &mut HashSet<BasicBlockId>,
    ) -> Option<BasicBlockId> {
        loop {
            if visited.contains(&active_block) {
                return Some(active_block);
            }
            visited.insert(active_block);
            let mut successors = self.cfg.successors(active_block);
            match successors.len() {
                0 => {
                    // Reached the end without colliding - the collision will happen when
                    // traversing the other branch.
                    return None;
                }
                1 => {
                    // Not an interesting block - move on
                    active_block = successors.next().unwrap();
                }
                2 => {
                    // Branch start - fork the recursion
                    let mut visited_by_children = HashSet::new();
                    let left_collision =
                        self.analyze(successors.next().unwrap(), &mut visited_by_children);
                    let right_collision =
                        self.analyze(successors.next().unwrap(), &mut visited_by_children);
                    let collision = match (left_collision, right_collision) {
                        (Some(collision), None) | (None, Some(collision)) => collision,
                        (Some(_),Some(_))=> unreachable!("A collision on both branches indicates a loop"), 
                        _ => unreachable!(
                            "Until we support multiple returns, branches always re-converge. Once supported this case should return `None`"
                        ),
                    };
                    for collision_predecessor in self.cfg.predecessors(collision) {
                        assert!(self.dom_tree.dominates(active_block, collision_predecessor));
                    }
                    self.branch_ends.insert(active_block, collision);

                    // Continue forward from child branch end until parent branches reconnect too.
                    active_block = collision;
                }
                _ => unreachable!("A block never has more than two successors"),
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::ssa_refactor::{
        ir::{cfg::ControlFlowGraph, function::RuntimeType, map::Id, types::Type},
        opt::flatten_cfg::branch_analysis::analyze_branch_ends,
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
        let branch_ends = analyze_branch_ends(&cfg, function);
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
        analyze_branch_ends(&cfg, function);
    }
}
