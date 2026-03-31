use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests array element access by index code-gen for Brillig.
#[test]
fn brillig_array_get() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_get v1, index v0 -> u32
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[3] = const u32 10
     1: sp[4] = const u32 20
     2: sp[5] = const u32 30
     3: sp[6] = @1
     4: sp[7] = const u32 4
     5: @1 = u32 add @1, sp[7]
     6: sp[6] = indirect const u32 1
     7: sp[7] = u32 add sp[6], @2
     8: sp[9] = const u32 3
     9: sp[8] = u32 add @0, sp[9]
    10: sp[10] = const u32 3
    11: @3 = sp[8]
    12: @4 = sp[7]
    13: @5 = sp[10]
    14: call 0 // -> MemCopy
    15: sp[4] = u32 add sp[6], @2
    16: sp[3] = u32 add sp[4], sp[2]
    17: sp[3] = load sp[3]
    18: sp[2] = sp[3]
    19: return
    ");
}

// Tests setting an array element and retrieving it
#[test]
fn brillig_array_set() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_set v1, index v0, value u32 99
        v3 = array_get v2, index v0 -> u32
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[3] = const u32 10
     1: sp[4] = const u32 20
     2: sp[5] = const u32 30
     3: sp[6] = @1
     4: sp[7] = const u32 4
     5: @1 = u32 add @1, sp[7]
     6: sp[6] = indirect const u32 1
     7: sp[7] = u32 add sp[6], @2
     8: sp[9] = const u32 3
     9: sp[8] = u32 add @0, sp[9]
    10: sp[10] = const u32 3
    11: @3 = sp[8]
    12: @4 = sp[7]
    13: @5 = sp[10]
    14: call 0 // -> MemCopy
    15: sp[3] = const u32 99
    16: @3 = sp[6]
    17: @4 = const u32 4
    18: call 0 // -> ArrayCopy
    19: sp[4] = @5
    20: sp[5] = u32 add sp[4], @2
    21: sp[7] = u32 add sp[5], sp[2]
    22: store sp[3] at sp[7]
    23: sp[5] = u32 add sp[4], @2
    24: sp[3] = u32 add sp[5], sp[2]
    25: sp[3] = load sp[3]
    26: sp[2] = sp[3]
    27: return
    ");
}

// Tests array operations with reference counting inc_rc
#[test]
fn brillig_array_with_rc_ops() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        inc_rc v0
        v1 = array_set v0, index u32 0, value u32 99
        v2 = array_get v1, index u32 0 -> u32
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[2] = const u32 10
     1: sp[3] = const u32 20
     2: sp[4] = const u32 30
     3: sp[5] = @1
     4: sp[6] = const u32 4
     5: @1 = u32 add @1, sp[6]
     6: sp[5] = indirect const u32 1
     7: sp[6] = u32 add sp[5], @2
     8: sp[8] = const u32 2
     9: sp[7] = u32 add @0, sp[8]
    10: sp[9] = const u32 3
    11: @3 = sp[7]
    12: @4 = sp[6]
    13: @5 = sp[9]
    14: call 0 // -> MemCopy
    15: sp[2] = load sp[5]
    16: sp[2] = u32 add sp[2], @2
    17: store sp[2] at sp[5]
    18: sp[2] = const u32 0
    19: sp[3] = const u32 99
    20: @3 = sp[5]
    21: @4 = const u32 4
    22: call 0 // -> ArrayCopy
    23: sp[4] = @5
    24: sp[6] = u32 add sp[4], sp[2]
    25: store sp[3] at sp[6]
    26: sp[3] = u32 add sp[4], sp[2]
    27: sp[3] = load sp[3]
    28: sp[2] = sp[3]
    29: return
    ");
}

// Regression test: global array passed as jmp argument to a single-predecessor block
// must not be param-side coalesced. Globals are allocated in a separate globals map,
// not in ssa_value_allocations, so fetching the "coalesced" value would fail.
//
// The DFG's indexing transparently resolves Value::Global to its underlying
// instruction in the globals graph, so the coalescing code must add special handling for globals.
#[test]
fn brillig_global_array_not_coalesced_with_block_param() {
    let src = "
    g0 = make_array [u8 65] : [u8; 1]

    brillig(inline) impure fn main f0 {
      b0(v0: [u8; 1]):
        v1 = allocate -> &mut u32
        store u32 1 at v1
        v2 = call f1(v1) -> u1
        jmpif v2 then: b1(), else: b2()
      b1():
        constrain u1 0 == u1 1
        unreachable
      b2():
        jmp b3(g0)
      b3(v3: [u8; 1]):
        return v3
    }
    brillig(inline) impure fn func_3 f1 {
      b0(v0: &mut u32):
        v1 = load v0 -> u32
        v2 = eq v1, u32 0
        jmpif v2 then: b1(), else: b2()
      b1():
        jmp b3(u1 0)
      b2():
        v3 = sub v1, u32 1
        store v3 at v0
        v4 = call f1(v0) -> u1
        v5 = not v4
        jmp b3(v5)
      b3(v6: u1):
        return v6
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    // Key opcodes:
    //   13: sp[2] = @68  — global g0 lives in global register @68, copied into sp[2] (param v3's slot)
    //   15: return        — returns sp[1]; global and param use separate allocations (not coalesced)
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[3] = @1
     1: @1 = u32 add @1, @2
     2: store @70 at sp[3]
     3: sp[5] = const u32 6
     4: sp[6] = @0
     5: sp[8] = sp[3]
     6: @0 = u32 add @0, sp[5]
     7: call 0 // -> f1
     8: @0 = sp[0]
     9: sp[4] = sp[8]
    10: jump if sp[4] to 0 // -> 15: f0/b1
    11: jump to 0 // -> 12: f0/b2
    12: sp[2] = @68 // f0/b2
    13: jump to 0 // -> 14: f0/b3
    14: return // f0/b3
    15: sp[2] = const bool 1 // f0/b1
    16: sp[3] = bool eq @69, sp[2]
    17: jump if sp[3] to 0 // -> 20: f0/b1/1
    18: sp[4] = const u32 0
    19: trap @[@1; sp[4]]
    ");
}
