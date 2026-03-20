use acvm::FieldElement;

use crate::brillig::brillig_gen::tests::{
    execute_brillig_from_ssa, execute_brillig_from_ssa_with_options,
};
use crate::brillig::{
    BrilligOptions,
    brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
};
use crate::ssa::ir::value::ValueId;
use crate::ssa::ssa_gen::Ssa;

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

/// This test is here to protect against the following scenario:
///
/// > In the `get_coalesced` fast path, `define_variable` reuses the partner’s cached register,
/// > records the new SSA value in `ssa_value_allocations`, marks it as available, and returns immediately.
///
/// > However, this path does not notify the active register allocator that the reused register is live again.
/// > If the partner value died earlier in the same block, its register may already have been deallocated and placed back into the allocator’s free pool.
/// > In that case, the allocator still considers the register available for reuse even though the coalesced value now relies on it.
///
/// > A later allocation can therefore hand out the same register to a different live value, causing register aliasing.
///
/// If the parameter was allowed to die and its register deallocated, the register could have already been allocated to something else,
/// before the coalescing partner is defined. `BrilligBlock` avoids deallocating registers prematurely by calling `CoalescingMap::has_live_partner`,
/// but this assumes that there *is* a live partner at the time. If the parameter died when none of its partners were defined yet,
/// it could happen, but we assume that if we define an `arg` which we want to pass to a `param`, then `param` should still be available at that point.
///
/// The `does_not_coalesce_param_side_when_arg_live_in_dest` verifies that we don't coalesce an `arg->param` if the
/// `arg` has been defined earlier than the `param`, which means `arg` is live where the `param` is defined. This is
/// why `define_variable` panics if `param` hasn't been defined yet.
///
/// In this test we artificially remove the `param`, and check that this causes a panic, so that if in the future
/// these assumptions would no longer hold, the fact that this test fails should mean the a regression should be
/// caught as well.
#[test]
#[should_panic(expected = "Coalesced parameter not currently available")]
fn coalescing_arg_to_deallocated_parameter_panics() {
    use crate::brillig::brillig_gen::brillig_block_variables::BlockVariables;
    use crate::brillig::{BrilligContext, FunctionContext};
    use crate::ssa::ir::instruction::TerminatorInstruction;

    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        jmp b1(u32 0)
      b1(v1: u32):
        v2 = lt v1, u32 3
        jmpif v2 then: b2(), else: b3()
      b2():
        v3 = add v1, u32 1
        v4 = mul v3, u32 2
        jmp b1(v4)
      b3():
        return v1
    }
    ";

    let ssa = Ssa::from_str(src).unwrap();
    let func = ssa.main();

    let blocks: Vec<_> = func.reachable_blocks().into_iter().collect();
    assert_eq!(blocks[2].to_u32(), 2, "should be b2");

    let block = &func.dfg[blocks[2]];
    let Some(TerminatorInstruction::Jmp { destination, arguments, .. }) = block.terminator() else {
        unreachable!("b2 ends in a jump to b1");
    };
    let arg = arguments[0]; // v4
    let param = func.dfg[*destination].parameters()[0]; // v1

    let options = BrilligOptions::default();
    let mut function_context =
        FunctionContext::new(func, true, options.layout.max_stack_frame_size());

    assert_eq!(
        function_context.coalescing.get_coalesced(&arg),
        Some(param),
        "v4 should coalesce to v1"
    );

    let brillig_context = BrilligContext::new("test", &options);
    let mut variables = BlockVariables::default();

    // Define the param SSA variable.
    let param_var =
        variables.define_variable(&mut function_context, &brillig_context, param, &func.dfg);

    // Remove the param SSA variable.
    // This should *not* happen before the arg is defined, under normal circumstances, but this test forces it!
    variables.remove_variable(&param, &function_context, &brillig_context);

    // Now define some other SSA variable, and see that it gets the same memory.
    let other = ValueId::new(3);
    let other_var =
        variables.define_variable(&mut function_context, &brillig_context, other, &func.dfg);
    assert_eq!(
        other_var.extract_register(),
        param_var.extract_register(),
        "freed register should be reallocated to new variable"
    );

    // Finally define the arg.
    // This should either fail, or allocate a different register.
    let arg_var =
        variables.define_variable(&mut function_context, &brillig_context, arg, &func.dfg);

    // If we allocated the same register than this should cause the test to fail.
    assert_ne!(
        arg_var.extract_register(),
        other_var.extract_register(),
        "the arg->param register is aliased with the other variable"
    );
}
