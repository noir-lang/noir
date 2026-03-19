use acvm::FieldElement;

use crate::brillig::brillig_gen::tests::{
    execute_brillig_from_ssa, execute_brillig_from_ssa_with_options,
};
use crate::brillig::{
    BrilligOptions,
    brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
};

/// Regression test from fuzzer: arg-side coalesced value (v1) outlives the block
/// param (v4) it shares a register with. When v4 dies in b1, the register was
/// incorrectly deallocated even though v1 is still used in b3. The register then
/// gets reassigned with a different bit_size, producing "Bit size for lhs N does
/// not match op bit size M" in the VM.
///
/// Key pattern: v1 (u32) is arg-coalesced to v4 (u32 block param of b1). In b1,
/// v4 dies after its use in `lt`, but v1 is still needed in b3. Operations after
/// v4's death in b1 allocate new values that reuse v4's freed register, corrupting
/// the value v1 still points to.
#[test]
fn coalescing_arg_outlives_param() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u8):
        v1 = cast v0 as u32
        v2 = cast v0 as u32
        jmp b1(v1)
      b1(v4: u32):
        v5 = lt v4, v2
        v6 = cast v0 as u8
        v7 = lt v6, u8 128
        jmpif v5 then: b2(), else: b3()
      b2():
        jmp b3()
      b3():
        v8 = unchecked_add v1, v2
        return v8
    }
    ";
    // v0=5, v1=5(u32), v2=5(u32)
    // b1: v4=5, 5<5=false → b3
    // b3: v8 = 5 + 5 = 10
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(5u64)]);
    assert_eq!(result, vec![FieldElement::from(10u64)]);
}

/// Param-side coalescing: block param v3 reuses v0's register (passthrough from b0 to b1).
/// v0 is then used in b2 after v3 dies. Tests the symmetric case of the register
/// deallocation bug where the param dies before the arg.
#[test]
fn coalescing_param_side_arg_outlives_param() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        v1 = unchecked_add v0, u32 10
        jmp b1(v0)
      b1(v3: u32):
        v4 = lt v3, v1
        v5 = unchecked_add v1, u32 1
        v6 = unchecked_add v5, u32 2
        jmpif v4 then: b2(), else: b3()
      b2():
        v7 = unchecked_add v0, v1
        jmp b4(v7)
      b3():
        jmp b4(u32 0)
      b4(v8: u32):
        return v8
    }
    ";
    // v0=5, v1=15
    // b1: v3=5, 5<15=true → b2
    // b2: v7 = 5 + 15 = 20
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(5u64)]);
    assert_eq!(result, vec![FieldElement::from(20u64)]);
}

/// Multiple coalescing pairs in the same jmp: both args outlive their params.
/// Tests that the fix handles multiple coalesced pairs correctly.
#[test]
fn coalescing_multiple_pairs_both_outlive() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        v1 = unchecked_add v0, u32 1
        v2 = unchecked_add v0, u32 2
        jmp b1(v1, v2)
      b1(v3: u32, v4: u32):
        v5 = unchecked_add v3, v4
        jmpif u1 1 then: b2(), else: b3()
      b2():
        v6 = unchecked_add v1, v2
        jmp b4(v6)
      b3():
        jmp b4(u32 0)
      b4(v7: u32):
        return v7
    }
    ";
    // v0=10, v1=11, v2=12
    // b1: v3=11, v4=12, v5=23 → b2
    // b2: v6 = 11 + 12 = 23
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(10u64)]);
    assert_eq!(result, vec![FieldElement::from(23u64)]);
}

/// Regression test for the interaction between arg-side coalescing and spilling.
///
/// # Bug
///
/// `FunctionContext::new` disables coalescing when spilling is needed
/// (see the comment "Disable coalescing when spilling is enabled") because
/// the two mechanisms interact unsafely:
///
/// When a successor block param (`v4`) is eagerly spilled in `convert_block_params`,
/// its register (`R`) is freed and returned to the allocator's free pool. An
/// instruction result that is arg-side coalesced with `v4` (`v2 -> v4`) reuses `R`
/// by reading `ssa_value_allocations[v4]` — but crucially it does **not** remove
/// `R` from the free pool, since the coalescing path bypasses `allocate_register`.
///
/// The freed register `R` is therefore still available for the next allocation.
/// When the subsequent instruction (`v3`) calls `allocate_register`, it pops `R`
/// from the free pool and also maps to `R`. Both `v2` and `v3` now alias the same
/// register. `v3`'s computation overwrites `R`, so the jmp that passes `v2` to
/// `b1` actually delivers `v3`'s value instead.
///
/// # Scenario (stack size 5 → 3 usable slots sp[2..4])
///
/// ```text
/// convert_block_params(b0):
///   define v4 → sp[4]; eagerly spill → sp[4] freed; pool = {sp[4]}
///
/// define v2 (coalesced with v4):
///   ssa_value_allocations[v2] = sp[4]  (sp[4] stays in free pool!)
///   compute: sp[4] = v0 + v1 = 10
///
/// define v3 (no coalescing):
///   allocate_register() pops sp[4] from pool → v3 = sp[4]  ← collision!
///   compute: sp[4] = v1 * v0 = 21
///
/// jmp b1(v2): reads sp[4] = 21 → writes 21 to v4's spill slot
/// b1: loads v4 from spill slot = 21, returns 21   ← wrong!
/// ```
///
/// # Current protection
///
/// `FunctionContext::new` sets `coalescing = CoalescingMap::default()` (empty)
/// whenever `spill_manager.is_some()`. This test therefore currently **passes**
/// (coalescing is a no-op when spilling is active).
///
/// The test is designed to **fail** if that guard is removed without a proper fix,
/// serving as the regression anchor for the future proper fix.
#[test]
fn coalescing_spill_arg_register_aliased_by_subsequent_allocation() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = unchecked_add v0, v1
        v3 = unchecked_mul v1, v0
        jmp b1(v2)
      b1(v4: u32):
        return v4
    }
    ";
    // v0=3, v1=7 → v2 = 3+7 = 10 (correct), v3 = 7*3 = 21 (the wrong value)
    // Without the fix: b1 returns 21 instead of 10.
    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(3u64), FieldElement::from(7u64)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(10u64)]);
}
