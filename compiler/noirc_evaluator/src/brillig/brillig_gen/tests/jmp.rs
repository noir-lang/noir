use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

/// A block that jumps to itself rotating its own parameters
/// (`jmp b1(v5, v6, v7, v4)` where `b1(v4, v5, v6, v7)`) forms a cycle in the
/// parameter-passing parallel move: every source register is also a destination
/// register. The block-argument mover must break the cycle with temporaries.
///
/// This exercises how many temporaries and move opcodes that cycle costs. A
/// general cycle-detecting solver rotates an N-cycle with a single temporary and
/// N+1 moves; an inline "copy every source-that-is-a-destination to a fresh temp"
/// strategy uses N temporaries and 2N moves.
#[test]
fn brillig_jmp_rotates_block_params_through_a_cycle() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32, v3: u32):
        jmp b1(v0, v1, v2, v3)
      b1(v4: u32, v5: u32, v6: u32, v7: u32):
        v8 = lt v4, v5
        jmpif v8 then: b2(), else: b3()
      b2():
        jmp b1(v5, v6, v7, v4)
      b3():
        return v4
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];

    // The rotation at `b2` (bytecode indices 8-15) copies all four sources into
    // fresh temporaries (sp[2..5]) before writing the destinations: 8 moves, 4
    // temporaries for a single 4-cycle.
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[6] = sp[2]
     1: sp[7] = sp[3]
     2: sp[8] = sp[4]
     3: sp[9] = sp[5]
     4: jump to 0 // -> 5: f0/b1
     5: sp[2] = u32 lt sp[6], sp[7] // f0/b1
     6: jump if sp[2] to 0 // -> 8: f0/b2
     7: jump to 0 // -> 17: f0/b3
     8: sp[2] = sp[7] // f0/b2
     9: sp[3] = sp[8]
    10: sp[4] = sp[9]
    11: sp[5] = sp[6]
    12: sp[6] = sp[2]
    13: sp[7] = sp[3]
    14: sp[8] = sp[4]
    15: sp[9] = sp[5]
    16: jump to 0 // -> 5: f0/b1
    17: sp[2] = sp[6] // f0/b3
    18: return
    ");
}
