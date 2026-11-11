//! Lowering of `jmp` and `jmpif` terminators.
//!
//! Blocks are laid out in reverse post order, so a terminator's destination is
//! often the block emitted right after it. Control falls through into that block,
//! so no `jump` opcode is emitted for it.

use acvm::FieldElement;

use crate::{
    assert_artifact_snapshot,
    brillig::brillig_gen::tests::{execute_brillig_from_ssa, ssa_to_brillig_artifacts},
    ssa::ir::map::Id,
};

const IF_ELSE_JOIN: &str = "
    brillig(inline) fn main f0 {
      b0(v0: u1, v1: u32):
        jmpif v0 then: b1(), else: b2()
      b1():
        v2 = add v1, u32 1
        jmp b3(v2)
      b2():
        v3 = add v1, u32 2
        jmp b3(v3)
      b3(v4: u32):
        return v4
    }
    ";

/// Blocks are laid out as `b0, b1, b2, b3`. `b2` falls through into the join block
/// `b3`. The jumps from `b0` to `b2` and from `b1` to `b3` skip over a block, so they
/// are kept.
#[test]
fn if_else_join() {
    let brillig = ssa_to_brillig_artifacts(IF_ELSE_JOIN);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
     0: jump if sp[2] to 0 // -> 2: f0/b1
     1: jump to 0 // -> 8: f0/b2
     2: sp[2] = const u32 1 // f0/b1
     3: sp[4] = u32 add sp[3], sp[2]
     4: sp[5] = u32 lt_eq sp[3], sp[4]
     5: jump if sp[5] to 0 // -> 7: f0/b1/1
     6: call 0 // -> ErrorWithString
     7: jump to 0 // -> 13: f0/b3
     8: sp[2] = const u32 2 // f0/b2
     9: sp[4] = u32 add sp[3], sp[2]
    10: sp[5] = u32 lt_eq sp[3], sp[4]
    11: jump if sp[5] to 0 // -> 13: f0/b2/1
    12: call 0 // -> ErrorWithString
    13: sp[2] = sp[4] // f0/b2/1, f0/b3
    14: return
    ");
}

#[test]
fn if_else_join_executes() {
    let then_result = execute_brillig_from_ssa(
        IF_ELSE_JOIN,
        vec![FieldElement::from(1u32), FieldElement::from(10u32)],
    );
    assert_eq!(then_result, vec![FieldElement::from(11u32)]);

    let else_result = execute_brillig_from_ssa(
        IF_ELSE_JOIN,
        vec![FieldElement::from(0u32), FieldElement::from(10u32)],
    );
    assert_eq!(else_result, vec![FieldElement::from(12u32)]);
}

const LOOP: &str = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        jmp b1(u32 0, u32 0)
      b1(v1: u32, v2: u32):
        v3 = lt v1, v0
        jmpif v3 then: b2(), else: b3()
      b2():
        v4 = add v2, v1
        v5 = unchecked_add v1, u32 1
        jmp b1(v5, v4)
      b3():
        return v2
    }
    ";

/// The preheader `b0` falls through into the loop header `b1`. The back edge
/// from `b2` to `b1` targets an earlier block, so its jump is kept.
#[test]
fn loop_preheader_and_back_edge() {
    let brillig = ssa_to_brillig_artifacts(LOOP);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[5] = const u32 0
     1: sp[6] = const u32 1
     2: sp[3] = sp[5]
     3: sp[4] = sp[5]
     4: sp[5] = u32 lt sp[3], sp[2] // f0/b1
     5: jump if sp[5] to 0 // -> 7: f0/b2
     6: jump to 0 // -> 15: f0/b3
     7: sp[5] = u32 add sp[4], sp[3] // f0/b2
     8: sp[7] = u32 lt_eq sp[4], sp[5]
     9: jump if sp[7] to 0 // -> 11: f0/b2/1
    10: call 0 // -> ErrorWithString
    11: sp[7] = u32 add sp[3], sp[6] // f0/b2/1
    12: sp[3] = sp[7]
    13: sp[4] = sp[5]
    14: jump to 0 // -> 4: f0/b1
    15: sp[2] = sp[4] // f0/b3
    16: return
    ");
}

#[test]
fn loop_executes() {
    // 0 + 1 + 2 + 3 + 4
    let result = execute_brillig_from_ssa(LOOP, vec![FieldElement::from(5u32)]);
    assert_eq!(result, vec![FieldElement::from(10u32)]);

    let result = execute_brillig_from_ssa(LOOP, vec![FieldElement::from(0u32)]);
    assert_eq!(result, vec![FieldElement::from(0u32)]);
}

const LOOP_EXIT_ON_THEN: &str = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        jmp b1(u32 0, u32 0)
      b1(v1: u32, v2: u32):
        v3 = eq v1, v0
        jmpif v3 then: b3(), else: b2()
      b2():
        v4 = add v2, v1
        v5 = unchecked_add v1, u32 1
        jmp b1(v5, v4)
      b3():
        return v2
    }
    ";

/// The loop header `b1` exits through its then-destination `b3` and falls through
/// into its else-destination `b2`, so only the conditional jump is emitted.
#[test]
fn jmpif_else_destination_is_next_block() {
    let brillig = ssa_to_brillig_artifacts(LOOP_EXIT_ON_THEN);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[5] = const u32 0
     1: sp[6] = const u32 1
     2: sp[3] = sp[5]
     3: sp[4] = sp[5]
     4: sp[5] = u32 eq sp[3], sp[2] // f0/b1
     5: jump if sp[5] to 0 // -> 14: f0/b3
     6: sp[5] = u32 add sp[4], sp[3]
     7: sp[7] = u32 lt_eq sp[4], sp[5]
     8: jump if sp[7] to 0 // -> 10: f0/b2/1
     9: call 0 // -> ErrorWithString
    10: sp[7] = u32 add sp[3], sp[6] // f0/b2/1
    11: sp[3] = sp[7]
    12: sp[4] = sp[5]
    13: jump to 0 // -> 4: f0/b1
    14: sp[2] = sp[4] // f0/b3
    15: return
    ");
}

#[test]
fn jmpif_else_destination_is_next_block_executes() {
    let result = execute_brillig_from_ssa(LOOP_EXIT_ON_THEN, vec![FieldElement::from(5u32)]);
    assert_eq!(result, vec![FieldElement::from(10u32)]);

    let result = execute_brillig_from_ssa(LOOP_EXIT_ON_THEN, vec![FieldElement::from(0u32)]);
    assert_eq!(result, vec![FieldElement::from(0u32)]);
}

const BRANCH_OVER_EMPTY_BLOCKS: &str = "
    brillig(inline) fn main f0 {
      b0(v0: u1, v1: u32):
        jmpif v0 then: b1(), else: b4()
      b1():
        jmp b2()
      b2():
        jmp b3()
      b3():
        jmp b4()
      b4():
        v2 = add v1, u32 1
        return v2
    }
    ";

/// `b1`, `b2` and `b3` only jump onwards, so they emit no opcodes. Both of `b0`'s
/// destinations then start at the next opcode, and neither jump is needed.
#[test]
fn branch_over_empty_blocks() {
    let brillig = ssa_to_brillig_artifacts(BRANCH_OVER_EMPTY_BLOCKS);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
    0: sp[2] = const u32 1
    1: sp[4] = u32 add sp[3], sp[2]
    2: sp[5] = u32 lt_eq sp[3], sp[4]
    3: jump if sp[5] to 0 // -> 5: f0/b4/1
    4: call 0 // -> ErrorWithString
    5: sp[2] = sp[4] // f0/b4/1
    6: return
    ");
}

#[test]
fn branch_over_empty_blocks_executes() {
    for condition in [0u32, 1] {
        let result = execute_brillig_from_ssa(
            BRANCH_OVER_EMPTY_BLOCKS,
            vec![FieldElement::from(condition), FieldElement::from(41u32)],
        );
        assert_eq!(result, vec![FieldElement::from(42u32)], "condition = {condition}");
    }
}

/// The `JmpIf` condition `v2` is a parameter of its own then-destination `b1`, and
/// the then-edge passes it a new value. The conditional move into `v2`'s register
/// must not change the predicate used by the remaining then-moves and the `JumpIf`.
#[test]
fn brillig_jmpif_then_move_does_not_overwrite_condition() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u1, v1: u32):
        jmp b1(v0, v1)
      b1(v2: u1, v3: u32):
        v4 = unchecked_add v3, u32 10
        v5 = unchecked_add v3, u32 1
        v6 = not v2
        jmpif v2 then: b1(v6, v5), else: b2(v4)
      b2(v7: u32):
        return v7
    }
    ";

    // b1(true, 0) takes the then-edge to b1(false, 1), which returns 1 + 10.
    let result =
        execute_brillig_from_ssa(src, vec![FieldElement::from(true), FieldElement::from(0u32)]);
    assert_eq!(result, vec![FieldElement::from(11u32)]);
}
