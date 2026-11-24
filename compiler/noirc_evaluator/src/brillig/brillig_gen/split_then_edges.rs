//! Brillig lowers a `jmpif` as `JumpIf condition then_block` followed by the else-edge, so
//! only the else-edge can pass block arguments with ordinary moves: moves for the then-edge
//! would run whether or not the branch is taken. Before codegen, every `jmpif` whose then-edge
//! carries arguments is therefore rerouted through a new block that forwards them:
//!
//! ```text
//! b0():                                b0():
//!   jmpif v0 then: b1(v2), else: b2()    jmpif v0 then: b3(), else: b2()
//!                                      b3():
//!                                        jmp b1(v2)
//! ```
//!
//! The forwarding block is an ordinary block, so its `jmp` goes through the regular parameter
//! passing (parallel moves, spill-slot stores), and the `jmpif` reads its condition exactly once.

use std::borrow::Cow;

use crate::ssa::ir::{
    basic_block::BasicBlockId, function::Function, instruction::TerminatorInstruction,
};

/// Returns `func` with no `jmpif` then-edge carrying arguments, borrowing it unchanged when it
/// has none.
pub(crate) fn split_jmpif_then_edges(func: &Function) -> Cow<'_, Function> {
    let has_then_arguments = |func: &Function, block: BasicBlockId| {
        matches!(
            func.dfg[block].terminator(),
            Some(TerminatorInstruction::JmpIf { then_arguments, .. }) if !then_arguments.is_empty()
        )
    };

    let blocks = func.reachable_blocks();
    if !blocks.iter().any(|block| has_then_arguments(func, *block)) {
        return Cow::Borrowed(func);
    }

    let mut func = Function::clone_with_id(func.id(), func);
    for block in blocks {
        if !has_then_arguments(&func, block) {
            continue;
        }
        let forward = func.dfg.make_block();
        let TerminatorInstruction::JmpIf { then_destination, then_arguments, call_stack, .. } =
            func.dfg[block].unwrap_terminator_mut()
        else {
            unreachable!("checked above");
        };
        let jmp = TerminatorInstruction::Jmp {
            destination: std::mem::replace(then_destination, forward),
            arguments: std::mem::take(then_arguments),
            call_stack: *call_stack,
        };
        func.dfg[forward].set_terminator(jmp);
    }
    Cow::Owned(func)
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    use super::split_jmpif_then_edges;

    #[test]
    fn routes_then_arguments_through_a_forwarding_block() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            jmpif v0 then: b1(v1), else: b1(u32 0)
          b1(v2: u32):
            return v2
        }
        ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let split = split_jmpif_then_edges(ssa.main()).into_owned();
        *ssa.main_mut() = split;

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            jmpif v0 then: b2(), else: b1(u32 0)
          b1(v2: u32):
            return v2
          b2():
            jmp b1(v1)
        }
        ");
    }

    #[test]
    fn leaves_functions_without_then_arguments_borrowed() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            jmpif v0 then: b1(), else: b2(v1)
          b1():
            jmp b2(u32 0)
          b2(v2: u32):
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        assert!(matches!(split_jmpif_then_edges(ssa.main()), std::borrow::Cow::Borrowed(_)));
    }
}
