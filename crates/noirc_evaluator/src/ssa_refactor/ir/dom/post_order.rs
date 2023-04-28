use std::collections::HashSet;

use crate::ssa_refactor::ir::{basic_block::BasicBlockId, function::Function};

/// DFT stack state marker for computing the cfg postorder.
enum Visit {
    First,
    Last,
}

// Calculates the post-order of the
pub(super) fn compute_post_order(func: &Function) -> Vec<BasicBlockId> {
    let mut stack = vec![(Visit::First, func.entry_block())];
    let mut visited: HashSet<BasicBlockId> = HashSet::new();
    let mut post_order: Vec<BasicBlockId> = Vec::new();

    while let Some((visit, block_id)) = stack.pop() {
        match visit {
            Visit::First => {
                if !visited.contains(&block_id) {
                    // This is the first time we pop the block, so we need to scan its
                    // successors and then revisit it.
                    visited.insert(block_id);
                    stack.push((Visit::Last, block_id));
                    // Note: cranelift choses to instead iterate successors backwards for reasons
                    // that aren't yet clearly relevant to us:
                    // https://github.com/bytecodealliance/wasmtime/commit/8abfe928d6073d76ebd991a2e991bf8268b4e5a2
                    for successor_id in func.dfg[block_id].successors() {
                        if !visited.contains(&successor_id) {
                            // This not visited check would also be cover by the the next
                            // iteration, but checking here two saves an iteration per successor.
                            stack.push((Visit::First, successor_id))
                        }
                    }
                }
            }

            Visit::Last => {
                // We've finished all this node's successors.
                post_order.push(block_id);
            }
        }
    }
    post_order
}

#[cfg(test)]
mod tests {
    use crate::ssa_refactor::ir::{
        function::Function, instruction::TerminatorInstruction, map::Id, types::Type,
    };

    use super::compute_post_order;

    #[test]
    fn single_block() {
        let func_id = Id::test_new(0);
        let func = Function::new("func".into(), func_id);
        let post_order = compute_post_order(&func);
        assert_eq!(post_order, [func.entry_block()]);
    }

    #[test]
    fn arb_graph_with_unreachable() {
        // A → B   C
        // ↓ ↗ ↓   ↓
        // D ← E → F
        // Technically correct post-order:
        // D, F, E, B, A, C
        // Post-order for our purposes:
        // F, E, B, D, A
        // Differences:
        // - Since C is unreachable we don't want to include it
        // - Siblings are traversed "backwards" (i.e. "else" before "then") to simply to save on
        //   an iterator conversion. We could change this if we find a motivation.

        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block_a_id = func.entry_block();
        let block_b_id = func.dfg.make_block();
        let block_c_id = func.dfg.make_block();
        let block_d_id = func.dfg.make_block();
        let block_e_id = func.dfg.make_block();
        let block_f_id = func.dfg.make_block();

        // A → B   •
        // ↓
        // D   •   •
        let cond_a = func.dfg.add_block_parameter(block_a_id, Type::unsigned(1));
        func.dfg.set_block_terminator(
            block_a_id,
            TerminatorInstruction::JmpIf {
                condition: cond_a,
                // Ordered backwards for test
                then_destination: block_b_id,
                else_destination: block_d_id,
            },
        );
        //  •   B   •
        //  •   ↓   •
        //  •   E   •
        func.dfg.set_block_terminator(
            block_b_id,
            TerminatorInstruction::Jmp { destination: block_e_id, arguments: vec![] },
        );
        // •   •   •
        //
        // D ← E → F
        let cond_e = func.dfg.add_block_parameter(block_e_id, Type::unsigned(1));
        func.dfg.set_block_terminator(
            block_e_id,
            TerminatorInstruction::JmpIf {
                condition: cond_e,
                then_destination: block_d_id,
                else_destination: block_f_id,
            },
        );
        // •   B   •
        //   ↗
        // D   •   •
        func.dfg.set_block_terminator(
            block_d_id,
            TerminatorInstruction::Jmp { destination: block_b_id, arguments: vec![] },
        );
        // •   •   C
        // •   •   ↓
        // •   •   F
        func.dfg.set_block_terminator(
            block_c_id,
            TerminatorInstruction::Jmp { destination: block_f_id, arguments: vec![] },
        );

        let post_order = compute_post_order(&func);
        assert_eq!(post_order, [block_f_id, block_e_id, block_b_id, block_d_id, block_a_id]);
    }
}
