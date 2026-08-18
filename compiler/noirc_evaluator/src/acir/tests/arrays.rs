use acvm::{
    acir::circuit::{Opcode, opcodes::BlockId},
    assert_circuit_snapshot,
};

use crate::{
    acir::{
        AcirDynamicArray, Context, SharedContext,
        acir_context::BrilligStdLib,
        tests::{ssa_to_acir_program, try_ssa_to_acir},
        types::AcirValue,
    },
    brillig::{Brillig, BrilligOptions},
    ssa::{ir::value::ValueId, ssa_gen::Ssa},
};

#[test]
fn array_get_of_zero_length_element_emits_no_orphan_init() {
    // A dynamic `array_get` whose result is a zero-length nested array (`[u8; 0]`)
    // reads no memory slots, so ACIR gen must not initialize the source array's
    // block: an orphan `MemoryInit` with no linked read is rejected by
    // `acir_post_check` with
    // "ICE: memory blocks initialized without any linked read/write/Brillig use".
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v1 = make_array [] : [u8; 0]
        v2 = make_array [v1, u8 1, v1, u8 2] : [([u8; 0], u8); 2]
        v3 = array_get v2, index v0 -> [u8; 0]
        return
    }
    ";
    try_ssa_to_acir(src).expect("zero-length-element array_get should compile to ACIR");
}

#[test]
fn array_set_not_mutable() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = array_get v0, index v1 -> Field
        v4 = array_set v0, index v1, value v2
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // Note how the non-mutable array_set ends up using a different block (b1)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT b0 = [w0, w1, w2]
    READ w8 = b0[w3]
    INIT b1 = [w0, w1, w2]
    WRITE b1[w3] = w4
    ASSERT w9 = 0
    READ w10 = b1[w9]
    ASSERT w11 = 1
    READ w12 = b1[w11]
    ASSERT w13 = 2
    READ w14 = b1[w13]
    ASSERT w5 = w10
    ASSERT w6 = w12
    ASSERT w7 = w14
    ");
}

#[test]
fn array_set_mutable() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = array_get v0, index v1 -> Field
        v4 = array_set mut v0, index v1, value v2
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // Now how the mutable array_set ends up using the same block (b0)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT b0 = [w0, w1, w2]
    READ w8 = b0[w3]
    WRITE b0[w3] = w4
    ASSERT w9 = 0
    READ w10 = b0[w9]
    ASSERT w11 = 1
    READ w12 = b0[w11]
    ASSERT w13 = 2
    READ w14 = b0[w13]
    ASSERT w5 = w10
    ASSERT w6 = w12
    ASSERT w7 = w14
    ");
}

#[test]
fn does_not_generate_memory_blocks_without_dynamic_accesses() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            call f1(u32 2, v3)
            v7 = array_get v0, index u32 0 -> Field
            constrain v7 == Field 0
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: [Field]):
              return
          }
        ";
    let program = ssa_to_acir_program(src);

    // Check that no memory opcodes were emitted.
    assert_eq!(program.functions.len(), 1);
    for opcode in &program.functions[0].opcodes {
        assert!(!matches!(opcode, Opcode::MemoryInit { .. }));
    }
}

#[test]
fn constant_array_access_out_of_bounds() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v2 = make_array [Field 0, Field 1] : [Field; 2]
        v4 = array_get v2, index u32 5 -> Field
        constrain v4 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // We expect a constant array access that is out of bounds (OOB) to be deferred to the runtime.
    // This means memory checks will be laid down and array access OOB checks will be handled there.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ASSERT w0 = 0
    ASSERT w1 = 1
    INIT b0 = [w0, w1]
    ASSERT w2 = 5
    READ w3 = b0[w2]
    ASSERT w3 = 0
    ");
}

#[test]
fn constant_array_access_in_bounds() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v2 = make_array [Field 0, Field 1] : [Field; 2]
        v4 = array_get v2, index u32 0 -> Field
        constrain v4 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // We know the circuit above to be trivially true
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].opcodes.len(), 0);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ");
}

#[test]
fn constant_reads_on_parameter_array_avoid_memory_blocks() {
    // A parameter array that is only ever read at constant indices should be resolved
    // entirely against its `AcirValue::Array`. ACIR generation must not initialize a memory
    // block for it, nor emit any `READ`/`WRITE` memory operations: each read folds directly to
    // the corresponding input witness.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3]):
        v2 = array_get v0, index u32 0 -> Field
        v4 = array_get v0, index u32 2 -> Field
        v5 = add v2, v4
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_eq!(program.functions.len(), 1);
    assert!(
        !program.functions[0]
            .opcodes
            .iter()
            .any(|opcode| matches!(opcode, Opcode::MemoryInit { .. } | Opcode::MemoryOp { .. })),
        "constant reads on a parameter array should not generate a memory block, got opcodes:\n{:#?}",
        program.functions[0].opcodes
    );
}

#[test]
fn predicated_constant_index_set_folds_without_memory_block() {
    // An `array_set` at a known in-bounds constant index under a predicate can be resolved at
    // compile time: the stored element becomes `predicate * value + (1 - predicate) * old`, and a
    // later constant-index read of the result folds to that element. Neither the set nor the read
    // should require a memory block, so no `INIT`/`READ`/`WRITE` is emitted — the whole thing
    // collapses to arithmetic on the input witnesses.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u1):
        v3 = array_get v0, index u32 0 -> Field
        v5 = add v3, Field 1
        enable_side_effects v1
        v6 = array_set v0, index u32 0, value v5
        enable_side_effects u1 1
        v7 = array_get v6, index u32 0 -> Field
        return v7
    }
    ";
    let program = ssa_to_acir_program(src);

    // `w0` is the original `v0[0]`, `w3` the predicate. The folded result is `v1 * (v0[0] + 1) +
    // (1 - v1) * v0[0]` which simplifies to `v0[0] + v1`, returned directly with no memory ops.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: [w4]
    BLACKBOX::RANGE input: w3, bits: 1
    ASSERT w4 = w0 + w3
    ");
}

#[test]
fn disabled_out_of_bounds_read_resolves_without_memory_block() {
    // Regression test for an ICE in `acir_post_check`:
    //   "Read at constant in-bounds index on memory block ... which has no preceding write".
    //
    // The read's flattened offset is computed with `unchecked_mul` (`3000000000 * 2`, which
    // overflows `u32`), so it never reaches SSA constant folding and stays a non-constant index
    // that `handle_constant_index` cannot resolve. The access is guarded by a predicate that the
    // preceding `constrain v == 0` pins to a compile-time zero.
    //
    // On the runtime memory-op path that zero predicate gates the index down to `0`, leaving a
    // constant, in-bounds `READ` on a never-written block. ACIR gen must instead recognize the
    // access as disabled and resolve it directly, emitting no memory block at all.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = make_array [u32 1, u32 2, u32 3, u32 4] : [(u32, u32)]
        v4 = eq v0, u32 5
        enable_side_effects v4
        constrain v4 == u1 0
        v7 = unchecked_mul u32 3000000000, u32 2
        v8 = array_get v2, index v7 -> u32
        enable_side_effects u1 1
        v9 = cast v4 as u32
        v10 = unchecked_mul v9, v8
        v11 = unchecked_add v10, v0
        return v11
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_eq!(program.functions.len(), 1);
    assert!(
        !program.functions[0]
            .opcodes
            .iter()
            .any(|opcode| matches!(opcode, Opcode::MemoryInit { .. } | Opcode::MemoryOp { .. })),
        "a disabled out-of-bounds read should not generate a memory block, got opcodes:\n{:#?}",
        program.functions[0].opcodes
    );
}

#[test]
fn disabled_out_of_bounds_read_from_global_resolves_without_memory_block() {
    // The same regression as `disabled_out_of_bounds_read_resolves_without_memory_block`, but
    // exercised from the exact final SSA the compiler emits for:
    //
    //   global G: [(u32, u32)] = [(1, 2), (3, 4)].as_vector();
    //   fn main(a: u32) -> pub u32 {
    //       if a == 5 { G[3000000000].0 } else { a }
    //   }
    //
    // The read on the global vector `g4` at the overflowing offset `3000000000 * 2` is guarded by a
    // predicate that `constrain 0 == v7` pins to a compile-time zero, so ACIR gen must resolve it as
    // a disabled access rather than materializing `g4` into a memory block.
    let src = "
    g0 = u32 1
    g1 = u32 2
    g2 = u32 3
    g3 = u32 4
    g4 = make_array [u32 1, u32 2, u32 3, u32 4] : [(u32, u32)]

    acir(inline) predicate_pure fn main f0 {
      b0(v5: u32):
        v7 = eq v5, u32 5
        enable_side_effects v7
        constrain u1 0 == v7, \"Index out of bounds\"
        v10 = unchecked_mul u32 3000000000, u32 2
        v11 = array_get g4, index v10 -> u32
        enable_side_effects u1 1
        v13 = cast v7 as u32
        v14 = unchecked_mul v13, v11
        v15 = unchecked_add v14, v5
        return v15
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_eq!(program.functions.len(), 1);
    assert!(
        !program.functions[0]
            .opcodes
            .iter()
            .any(|opcode| matches!(opcode, Opcode::MemoryInit { .. } | Opcode::MemoryOp { .. })),
        "a disabled out-of-bounds read should not generate a memory block, got opcodes:\n{:#?}",
        program.functions[0].opcodes
    );
}

#[test]
fn disabled_out_of_bounds_read_with_predicate_restored_resolves_without_memory_block() {
    // Minimized from `orig_vs_morph` fuzzer seed 0xa7d9bbfc0000a00d.
    //
    // The distinguishing feature versus `disabled_out_of_bounds_read_from_global_resolves_without_memory_block`
    // is that the `enable_side_effects` guarding the read has been *restored to a constant one*
    // (`enable_side_effects u1 1`) before the `array_get`: the read is not syntactically under the
    // pinned predicate `v7`. Instead the block is dead because two constraints on `v7` contradict
    // (`constrain 0 == v7` then `constrain v7 == 1`), which ACIR gen resolves to a compile-time zero
    // side-effects predicate. So a pass keyed only on the predicate a `constrain x == 0` pins would
    // miss this read, but the ACIR-gen check on the effective side-effects predicate still resolves
    // it as disabled and emits no memory block for the overflowing (`3000000000 * 2`) index.
    let src = "
    g0 = u32 1
    g1 = u32 2
    g2 = u32 3
    g3 = u32 4
    g4 = make_array [u32 1, u32 2, u32 3, u32 4] : [(u32, u32)]

    acir(inline) predicate_pure fn main f0 {
      b0(v5: u32):
        v7 = eq v5, u32 5
        enable_side_effects v7
        constrain u1 0 == v7, \"Index out of bounds\"
        enable_side_effects u1 1
        constrain v7 == u1 1, \"Index out of bounds\"
        v10 = unchecked_mul u32 3000000000, u32 2
        v11 = array_get g4, index v10 -> u32
        return v11
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_eq!(program.functions.len(), 1);
    assert!(
        !program.functions[0]
            .opcodes
            .iter()
            .any(|opcode| matches!(opcode, Opcode::MemoryInit { .. } | Opcode::MemoryOp { .. })),
        "a disabled out-of-bounds read should not generate a memory block, got opcodes:\n{:#?}",
        program.functions[0].opcodes
    );
}

#[test]
fn safe_index_read_under_disabled_predicate_emits_the_read() {
    // The side-effects latch is only the reading instruction's own predicate for instructions
    // that `remove_enable_side_effects` fences, i.e. those reporting
    // `requires_acir_gen_predicate == true`. A safe-index `array_get` reports `false`, so the
    // pass is free to move an `EnableSideEffectsIf` past it and the latch reaching ACIR gen can
    // belong to an entirely different region.
    //
    // The SSA below is the shape that pass leaves behind: `v3` is pinned to a compile-time zero
    // by `constrain u1 0 == v3`, but the read at the constant index `0` belongs to the region
    // *after* it and is live. Resolving it as a disabled access would bind a live value to zero
    // and delete the `constrain` the program wrote over it.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 4], v1: u32, v2: Field, v3: u1):
        v4 = array_set v0, index v1, value v2
        enable_side_effects v3
        constrain u1 0 == v3, \"bad branch\"
        v5 = array_get v4, index u32 0 -> Field
        enable_side_effects u1 1
        constrain v5 != Field 999
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    // `READ w9 = b0[w8]` (with `w8` pinned to the constant index `0`) is the read the program
    // asked for, `w7 = w9` returns it rather than a zero, and the `!= 999` constraint the source
    // wrote over it survives as the `directive_invert` call and its inverse constraint.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6]
    public parameters: []
    return values: [w7]
    INIT b0 = [w0, w1, w2, w3]
    WRITE b0[w4] = w5
    ASSERT w6 = 0
    ASSERT w8 = 0
    READ w9 = b0[w8]
    BRILLIG CALL func: 0, predicate: 1, inputs: [w9 - 999], outputs: [w10]
    ASSERT 0 = w9*w10 - 999*w10 - 1
    ASSERT w7 = w9

    unconstrained func 0: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop @[@20; @21]
    ");
}

#[test]
fn unsafe_index_read_under_disabled_predicate_still_resolves_as_disabled() {
    // The boundary of the fix above: a read whose index is *not* statically safe does report
    // `requires_acir_gen_predicate == true`, so `remove_enable_side_effects` fences it and the
    // latch really is this read's own predicate. Such a read must keep taking the disabled
    // shortcut — no memory block, result zeroed.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 4], v1: u32, v2: Field, v3: u1):
        v4 = array_set v0, index v1, value v2
        enable_side_effects v3
        constrain u1 0 == v3, \"bad branch\"
        v5 = array_get v4, index v1 -> Field
        enable_side_effects u1 1
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6]
    public parameters: []
    return values: [w7]
    INIT b0 = [w0, w1, w2, w3]
    WRITE b0[w4] = w5
    ASSERT w6 = 0
    ASSERT w7 = 0
    ");
}

#[test]
fn generates_memory_op_for_dynamic_read() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = array_get v0, index v1 -> Field
        constrain v2 == Field 10
        return
    }
    ";

    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: []
    INIT b0 = [w0, w1, w2]
    READ w4 = b0[w3]
    ASSERT w4 = 10
    ");
}

#[test]
fn generates_memory_op_for_dynamic_write() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = array_set v0, index v1, value Field 10
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // All logic after the write is expected as we generate new witnesses for return values
    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: [w4, w5, w6]
    INIT b0 = [w0, w1, w2]
    ASSERT w7 = 10
    WRITE b0[w3] = w7
    ASSERT w8 = 0
    READ w9 = b0[w8]
    ASSERT w10 = 1
    READ w11 = b0[w10]
    ASSERT w12 = 2
    READ w13 = b0[w12]
    ASSERT w4 = w9
    ASSERT w5 = w11
    ASSERT w6 = w13
    ");
}

// An array index can be a compile-time constant to ACIR gen while remaining symbolic in SSA:
// `dfg.get_numeric_constant` answers for SSA, but ACIR gen also folds over its own expression
// algebra. A read at such an index must still take the compile-time path — laying it down as a
// memory op leaves a `READ` at a constant index on a block that is never written, which is a
// fully determined read and is what `assert_constant_reads_are_folded` rejects.
//
// The tests below cover each route by which an index reaches ACIR gen in that state, plus the
// cases that must keep their memory block.

#[test]
fn folds_read_at_an_index_constrained_from_a_parameter() {
    // Lowering `constrain u1 0 == v0` rewrites `v0`'s `AcirVar` to the constant `0` (see
    // `AcirContext::mark_variables_equivalent`, which prefers a constant over a witness). The
    // `cast` and `unchecked_add` above it then fold, leaving the index the constant `2`.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = make_array [u1 0, u1 0, u1 1] : [u1; 3]
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 2
        v4 = array_get v1, index v3 -> u1
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // No `INIT`/`READ` pair at all: the array never reaches a memory block, and the returned
    // value is the constant `1` held at index 2.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 1
    ");
}

#[test]
fn keeps_memory_block_when_constrain_protects_an_input_witness() {
    // The mirror image of `folds_read_at_an_index_constrained_from_a_parameter`, differing only in
    // which side of the `constrain` the parameter sits. `mark_variables_equivalent` declines to
    // rewrite an input witness, and it inspects its `lhs` argument alone, so with the parameter on
    // the left no substitution happens and the index stays symbolic in ACIR too. The read is then
    // genuinely dynamic as far as ACIR gen can tell and keeps its memory block.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = make_array [u1 0, u1 0, u1 1] : [u1; 3]
        constrain v0 == u1 0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 2
        v4 = array_get v1, index v3 -> u1
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w2 = 0
    ASSERT w3 = 1
    INIT b0 = [w2, w2, w3]
    ASSERT w4 = w0 + 2
    READ w5 = b0[w4]
    ASSERT w1 = w5
    ");
}

#[test]
fn folds_read_at_an_index_cancelled_by_expression_algebra() {
    // The final SSA for `a[p0.wrapping_sub(p0)]`. There is no `constrain` here at all: the index
    // becomes constant purely because ACIR gen's polynomial arithmetic cancels the `v6` terms of
    // `(v6 + 2^128) - v6`. SSA's `sub x, x` identity does not match, since the operands are
    // distinct values.
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32):
        v5 = make_array [u8 10, u8 20, u8 30, u8 40] : [u8; 4]
        v6 = cast v0 as Field
        v8 = add v6, Field 340282366920938463463374607431768211456
        v9 = sub v8, v6
        v10 = truncate v9 to 32 bits, max_bit_size: 254
        v11 = cast v10 as u32
        v12 = array_get v5, index v11 -> u8
        return v12
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 32
    ASSERT w1 = 10
    ");
}

#[test]
fn folds_read_at_a_constrained_derived_index_with_value_on_the_left() {
    // `mark_variables_equivalent`'s input-witness guard only protects a parameter's own witness.
    // Once the constrained value is derived, the substitution happens whichever side it sits on,
    // so this and `..._with_value_on_the_right` must agree.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v1 = make_array [u8 10, u8 20, u8 30, u8 40] : [u8; 4]
        v2 = unchecked_add v0, u32 7
        constrain v2 == u32 7
        v3 = unchecked_sub v2, u32 5
        v4 = array_get v1, index v3 -> u8
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 30
    ");
}

#[test]
fn folds_read_at_a_constrained_derived_index_with_value_on_the_right() {
    // See `..._with_value_on_the_left`: the operand order is immaterial for a derived value.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v1 = make_array [u8 10, u8 20, u8 30, u8 40] : [u8; 4]
        v2 = unchecked_add v0, u32 7
        constrain u32 7 == v2
        v3 = unchecked_sub v2, u32 5
        v4 = array_get v1, index v3 -> u8
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 30
    ");
}

#[test]
fn folds_read_from_a_global_array_at_an_acir_only_constant_index() {
    // A global array is an `AcirValue::Array` like a local `make_array`, so it takes the same
    // compile-time path.
    let src = "
    g0 = make_array [u8 10, u8 20, u8 30, u8 40] : [u8; 4]

    acir(inline) fn main f0 {
      b0(v0: u1):
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 2
        v4 = array_get g0, index v3 -> u8
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 30
    ");
}

#[test]
fn folds_read_at_an_acir_only_constant_index_into_a_tuple_array() {
    // An array of tuples is flattened in SSA, so the index is scaled by the element width before
    // the read. The scaling multiplication folds along with everything else, landing on the
    // `Field` slot at flattened index 4.
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v5: u1):
        v0 = make_array [Field 7, u1 0, Field 8, u1 0, Field 9, u1 0, Field 10, u1 0] : [(Field, u1)]
        enable_side_effects v5
        constrain u1 0 == v5, \"Index out of bounds\"
        enable_side_effects u1 1
        v7 = cast v5 as u32
        v9 = unchecked_mul v7, u32 2329284907
        v11 = unchecked_add v9, u32 3320108434
        v12 = truncate v11 to 2 bits, max_bit_size: 32
        v15 = unchecked_mul v12, u32 2
        v16 = array_get v0, index v15 -> Field
        return v16
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 9
    ");
}

#[test]
fn folds_read_at_a_constant_slot_of_a_partly_witness_array() {
    // `assert_constant_reads_are_folded` exempts a read whose initialized value is a non-constant
    // witness, but it decides that per slot rather than per array. An array holding one witness
    // among constants is therefore still fully determined at a constant slot.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u8):
        v2 = make_array [v1, u8 20, u8 30, u8 40] : [u8; 4]
        constrain u1 0 == v0
        v3 = cast v0 as u32
        v4 = unchecked_add v3, u32 2
        v5 = array_get v2, index v4 -> u8
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w1, bits: 8
    ASSERT w0 = 0
    ASSERT w2 = 30
    ");
}

#[test]
fn folds_read_on_a_parameter_array_at_an_acir_only_constant_index() {
    // A parameter array is an `AcirValue::Array` whose elements are witnesses, so the fold yields
    // the witness at that slot rather than a literal, and the array still never needs a block.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1, v1: [u8; 4]):
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 2
        v4 = array_get v1, index v3 -> u8
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // `w1`..`w4` are the array's elements, so slot 2 is `w3`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5]
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    ASSERT w0 = 0
    ASSERT w5 = w3
    ");
}

#[test]
fn folds_write_at_an_acir_only_constant_index() {
    // The same gate gates `array_set`. A write at a known in-bounds slot under an enabled
    // predicate is resolved into the `AcirValue::Array`, so a later constant-index read of the
    // result sees the stored value without either operation touching memory.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = make_array [u8 10, u8 20, u8 30, u8 40] : [u8; 4]
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 2
        v4 = array_set v1, index v3, value u8 99
        v5 = array_get v4, index u32 2 -> u8
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w1 = 99
    ");
}

#[test]
fn defers_out_of_bounds_acir_only_constant_index_to_runtime() {
    // An out-of-bounds constant index has no initialized value to fold to, so it stays on the
    // memory path and fails at runtime, matching `constant_array_access_out_of_bounds`.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = make_array [u1 0, u1 0, u1 1] : [u1; 3]
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 9
        v4 = array_get v1, index v3 -> u1
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w2 = 0
    ASSERT w3 = 1
    INIT b0 = [w2, w2, w3]
    ASSERT w4 = 9
    READ w5 = b0[w4]
    ASSERT w1 = w5
    ");
}

#[test]
fn defers_acir_only_constant_index_equal_to_the_array_length_to_runtime() {
    // The boundary of `defers_out_of_bounds_acir_only_constant_index_to_runtime`: the first slot
    // past the end must be treated as out of bounds, not folded to the last element.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = make_array [u1 0, u1 0, u1 1] : [u1; 3]
        constrain u1 0 == v0
        v2 = cast v0 as u32
        v3 = unchecked_add v2, u32 3
        v4 = array_get v1, index v3 -> u1
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w0 = 0
    ASSERT w2 = 0
    ASSERT w3 = 1
    INIT b0 = [w2, w2, w3]
    ASSERT w4 = 3
    READ w5 = b0[w4]
    ASSERT w1 = w5
    ");
}

#[test]
fn generates_predicated_index_for_dynamic_read() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, predicate: bool):
        enable_side_effects predicate
        v3 = array_get v0, index v1 -> Field
        constrain v3 == Field 10
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // w0, w1, w2 represents the array
    // So w3 represents our index and w4 is our predicate
    // `ASSERT w5 = w3*w4` is the predicate gate inside `get_flattened_index`, which forces
    // the read to fall back to a safe in-bounds slot when the predicate is `0`. Since
    // `compute_offset` returns `Some(0)` for `[Field; 3]`, no offset-fallback bias is
    // applied and we read directly at `w5`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 1
    INIT b0 = [w0, w1, w2]
    ASSERT w5 = w3*w4
    READ w6 = b0[w5]
    ASSERT w6 = 10
    ");
}

#[test]
fn generates_predicated_index_and_dummy_value_for_dynamic_write() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, predicate: bool):
        enable_side_effects predicate
        v3 = array_set v0, index v1, value Field 10
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    // Similar to the `generates_predicated_index_for_dynamic_read` test, `w8 = w3*w4` is the
    // predicate gate inside `get_flattened_index`. `compute_offset` returns `Some(0)` here so
    // no offset-fallback bias is applied and we use `w8` directly.
    // We then have extra logic for generating a dummy value.
    // The original value we want to write is `Field 10` and our predicate is `w4`.
    // We read the value at the predicated index into `w9`. This is our dummy value.
    // We can then see how we form our new store value with:
    // `ASSERT -w4*w9 + 10*w4 + w9 - w10 = 0` -> (predicate*value + (1-predicate)*dummy)
    // `10*w4` -> predicate*value
    // `-w4*w9` -> (-predicate * dummy)
    // `w9` -> dummy
    // As expected, we then store `w10` at the predicated index `w8`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 1
    INIT b0 = [w0, w1, w2]
    ASSERT w8 = w3*w4
    READ w9 = b0[w8]
    INIT b1 = [w0, w1, w2]
    ASSERT w10 = -w4*w9 + 10*w4 + w9
    WRITE b1[w8] = w10
    ASSERT w11 = 0
    READ w12 = b1[w11]
    ASSERT w13 = 1
    READ w14 = b1[w13]
    ASSERT w15 = 2
    READ w16 = b1[w15]
    ASSERT w5 = w12
    ASSERT w6 = w14
    ASSERT w7 = w16
    ");
}

#[test]
fn zero_length_array_constant() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v0 = make_array [] : [Field; 0]
        v2 = array_get v0, index u32 0 -> Field
        constrain v2 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // As we have a constant array the constraint we insert will be simplified down.
    // We expect ever expression to equal zero when executed. Thus, this circuit will always fail.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ASSERT 0 = 1
    ");
}

#[test]
fn zero_length_array_dynamic_set() {
    // An array of zero-width elements (here `[u8; 0]`, the lowering of `str<0>`)
    // has a flattened size of zero, so each element flattens to zero numeric types.
    // A dynamic `array_set` must not divide by that empty `value_types` length.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v1 = make_array b\"\"
        v2 = make_array [v1, v1, v1, v1] : [[u8; 0]; 4]
        v3 = array_set v2, index v0, value v1
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    ");
}

#[test]
fn zero_length_array_is_not_initialized() {
    // A dynamic operation on an array whose flattened size is zero (`[[u8; 0]; 4]`, whose elements
    // are zero-width) must not emit a `MemoryInit` for the empty backing block. An empty block has
    // no slots to read or write, so any such `MemoryInit` describes an orphan block.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v1 = make_array b\"\"
        v2 = make_array [v1, v1, v1, v1] : [[u8; 0]; 4]
        v3 = array_set v2, index v0, value v1
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_eq!(program.functions.len(), 1);
    assert!(
        !program.functions[0]
            .opcodes
            .iter()
            .any(|opcode| { matches!(opcode, Opcode::MemoryInit { init, .. } if init.is_empty()) }),
        "ACIR gen must not initialize a zero-length memory block, got opcodes:\n{:#?}",
        program.functions[0].opcodes
    );
}

#[test]
fn zero_length_array_dynamic_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(predicate: bool):
        enable_side_effects predicate
        v0 = make_array [] : [Field; 0]
        v2 = array_get v0, index u32 0 -> Field
        constrain v2 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Similar to the `zero_length_array_constant` test we inserted an always failing constraint
    // when an array access is attempted on a zero length array.
    // However, we must gate it by the predicate in case the branch is inactive.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    ASSERT w0 = 0
    ");
}

/// Tests this code:
/// ```noir
/// struct Bar {
///     inner: [Field; 3],
/// }
/// struct Foo {
///     a: Field,
///     b: [Field; 3],
///     bar: Bar,
/// }
/// fn main(x: [Foo; 4], index: u32) -> pub [Field; 3] {
///     x[index].bar.inner
/// }
/// ```
#[test]
fn non_homogenous_array_dynamic_access() {
    let src = r#"
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [(Field, [Field; 3], [Field; 3]); 4], v1: u32):
        v2 = array_get v0, index v1 -> [Field; 3]
        return v2
    }
    "#;

    let program = ssa_to_acir_program(src);

    // b0 is our actual array input while b1 is our element type sizes array.
    // You can see that in `w44 = b1[w28]` we use the supplied witness index to read the flattened index from b1.
    // `w44` is then used to read from the b0 array.
    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28]
    public parameters: []
    return values: [w29, w30, w31]
    ASSERT w32 = 0
    ASSERT w33 = 1
    ASSERT w34 = 4
    ASSERT w35 = 7
    ASSERT w36 = 8
    ASSERT w37 = 11
    ASSERT w38 = 14
    ASSERT w39 = 15
    ASSERT w40 = 18
    ASSERT w41 = 21
    ASSERT w42 = 22
    ASSERT w43 = 25
    INIT b0 = [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43]
    READ w44 = b0[w28]
    INIT b1 = [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27]
    READ w45 = b1[w44]
    ASSERT w46 = w44 + 1
    READ w47 = b1[w46]
    ASSERT w48 = w46 + 1
    READ w49 = b1[w48]
    ASSERT w29 = w45
    ASSERT w30 = w47
    ASSERT w31 = w49
    ");
}

#[test]
fn make_dynamic_array_value_types() {
    let src = r#"
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [[([Field; 2], u8); 3]; 4], v1: u32, v2: [([Field; 2], u8); 3]):
        v3, v4 = call as_vector(v0) -> (u32, [[([Field; 2], u8); 3]])
        v5 = array_set v4, index v1, value v2
        return
    }
    "#;
    let ssa = Ssa::from_str(src).unwrap();
    let (_, main) = ssa.functions.iter().next().unwrap();

    // Create an empty context we can test.
    let mut shared_context = SharedContext::default();
    let brillig = Brillig::default();
    let brillig_options = BrilligOptions::default();
    let mut context =
        Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

    // Make sure all the values are cached, following a bit of how `convert_acir_main` would do it.
    let entry_block = &main.dfg[main.entry_block()];
    context.convert_ssa_block_params(entry_block.parameters(), &main.dfg).unwrap();
    for instruction_id in entry_block.instructions() {
        context.convert_ssa_instruction(*instruction_id, &main.dfg, &ssa).unwrap();
    }

    // Now repeat the step that generates the ACIR for the result of an array set.
    let array_id = ValueId::new(5);
    let array = context.make_array_set_result_value(array_id, BlockId::new(0), &main.dfg).unwrap();
    let AcirValue::DynamicArray(AcirDynamicArray { len, value_types, .. }) = array else {
        panic!("expected DynamicArray, got {array:?}");
    };
    assert_eq!(
        len.to_usize(),
        (2 + 1) * 3 * 4,
        "a vector should have all the nested arrays flattened into it, up to its capacity"
    );
    assert_eq!(
        value_types.len(),
        (2 + 1) * 3,
        "a vector should have all the types of its first element flattened"
    );
}

#[test]
fn predicated_composite_get_with_heterogeneous_element_layout() {
    // A dynamic `array_get` under a dynamic predicate, on an array whose element layout is
    // heterogeneous (`(u8, [(u8, Field); 2])` flattens to `u8, u8, Field, u8, Field`). The SSA
    // index `v1 * 2 + 1` targets the `[(u8, Field); 2]` field of element `v1`.
    //
    // On a disabled branch the read must not leave any result leaf holding a value wider than
    // its declared type: the leaves (declared `u8, Field, u8, Field`) would read the leading
    // flat slots of the array, and a `Field` slot landing under a `u8` leaf would produce an
    // unconstrained wide witness tagged as narrow.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [(u8, [(u8, Field); 2]); 3], v1: u32, v2: u1):
        enable_side_effects v2
        v4 = unchecked_mul v1, u32 2
        v6 = unchecked_add v4, u32 1
        v7 = array_get v0, index v6 -> [(u8, Field); 2]
        enable_side_effects u1 1
        return v7
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16]
    public parameters: []
    return values: [w17, w18, w19, w20]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w15, bits: 32
    BLACKBOX::RANGE input: w16, bits: 1
    ASSERT w21 = 0
    ASSERT w22 = 1
    ASSERT w23 = 5
    ASSERT w24 = 6
    ASSERT w25 = 10
    ASSERT w26 = 11
    INIT b0 = [w21, w22, w23, w24, w25, w26]
    ASSERT w27 = 2*w15*w16 + w16
    READ w28 = b0[w27]
    INIT b1 = [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14]
    ASSERT w29 = -w16 + w28 + 1
    READ w30 = b1[w29]
    ASSERT w31 = w29 + 1
    READ w32 = b1[w31]
    ASSERT w33 = w31 + 1
    READ w34 = b1[w33]
    ASSERT w35 = w33 + 1
    READ w36 = b1[w35]
    ASSERT w17 = w30
    ASSERT w18 = w32
    ASSERT w19 = w34
    ASSERT w20 = w36
    ");
}

#[test]
fn predicated_composite_get_on_vector_with_heterogeneous_items() {
    // Same hazard as `predicated_composite_get_with_heterogeneous_element_layout`, but on a
    // vector: the item sizes are not homogeneous (`Field` is 1 slot, `[u8; 2]` is 2), and the
    // read targets the `[u8; 2]` item, whose `u8` leaves must not be left holding the vector's
    // leading `Field` slot on a disabled branch.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [u8 2, u8 3] : [u8; 2]
        v7 = make_array [u8 5, u8 6] : [u8; 2]
        v8 = make_array [Field 1, v4, Field 4, v7] : [(Field, [u8; 2])]
        enable_side_effects v1
        v9 = unchecked_mul v0, u32 2
        v10 = unchecked_add v9, u32 1
        v11 = array_get v8, index v10 -> [u8; 2]
        enable_side_effects u1 1
        return v11
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2, w3]
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w4 = 0
    ASSERT w5 = 1
    ASSERT w6 = 3
    ASSERT w7 = 4
    INIT b0 = [w4, w5, w6, w7]
    ASSERT w8 = 2*w0*w1 + w1
    READ w9 = b0[w8]
    ASSERT w10 = 2
    ASSERT w11 = 5
    ASSERT w12 = 6
    INIT b1 = [w5, w10, w6, w7, w11, w12]
    ASSERT w13 = -w1 + w9 + 1
    READ w14 = b1[w13]
    ASSERT w15 = w13 + 1
    READ w16 = b1[w15]
    ASSERT w2 = w14
    ASSERT w3 = w16
    ");
}

#[test]
#[should_panic(expected = "is not a field of the array element type")]
fn type_mismatched_array_get_is_an_ice() {
    // An `array_get` whose result type is not one of the element's field types cannot come out
    // of SSA generation, so the disabled-branch fallback offset cannot be computed for it. The
    // SSA validator currently accepts such (hand-written) SSA, so ACIR gen refuses it with a
    // deliberate ICE rather than picking a fallback index whose slot type it cannot vouch for.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: u1):
        enable_side_effects v2
        v3 = array_get v0, index v1 -> u8
        enable_side_effects u1 1
        return v3
    }
    ";
    let _ = ssa_to_acir_program(src);
}

#[test]
fn predicated_get_of_scalar_field_biases_index_to_matching_slot() {
    // A dynamic get of the `u32` field of a `(Field, u32)` element under a dynamic predicate.
    // All items are single-slot, so the disabled-branch fallback index can be biased to slot 1
    // (`(1 - predicate) * 1`), whose type matches the result exactly — no masking needed.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [(Field, u32); 3], v1: u32, v2: u1):
        enable_side_effects v2
        v4 = unchecked_mul v1, u32 2
        v6 = unchecked_add v4, u32 1
        v7 = array_get v0, index v6 -> u32
        enable_side_effects u1 1
        return v7
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7]
    public parameters: []
    return values: [w8]
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w5, bits: 32
    BLACKBOX::RANGE input: w6, bits: 32
    BLACKBOX::RANGE input: w7, bits: 1
    INIT b0 = [w0, w1, w2, w3, w4, w5]
    ASSERT w9 = 2*w6*w7 + 1
    READ w10 = b0[w9]
    ASSERT w8 = w10
    ");
}

#[test]
fn predicated_constant_index_get_on_heterogeneous_vector() {
    // A *constant* index read, the counterpart of
    // `predicated_composite_get_on_vector_with_heterogeneous_items`.
    //
    // The fallback offset may only be added to an index that the predicate gates down to `0`.
    // For a layout whose fields have differing flattened sizes,
    // [`Context::get_flattened_index`] resolves a constant index through the element-type-sizes
    // table into a flat offset that is already known to be in bounds, and therefore returns it
    // ungated — and unbiased, even though `DataFlowGraph::is_safe_index` reports `false` for
    // every vector (it cannot see the vector's semantic length): a bias on top of that resolved
    // offset would relocate the read to the wrong slots, or off the end of the block, exactly
    // when the predicate is `0`.
    //
    // The vector holds two `(Field, [u8; 2])` items in six slots. Semi-flattened index `3` is
    // item 1's `[u8; 2]`, i.e. flat slot 4, and the read must stay on slots 4 and 5 whatever
    // the predicate is.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u1, v2: Field):
        v3 = make_array [u8 2, u8 3] : [u8; 2]
        v4 = make_array [u8 5, u8 6] : [u8; 2]
        v5 = make_array [Field 1, v3, Field 4, v4] : [(Field, [u8; 2])]
        v6 = array_set v5, index v0, value v2
        enable_side_effects v1
        v7 = array_get v6, index u32 3 -> [u8; 2]
        enable_side_effects u1 1
        return v7
    }
    ";
    let program = ssa_to_acir_program(src);
    // The index resolves to the constant `4` and both reads stay on slots 4 and 5 whatever the
    // predicate is — identical to the array-typed counterpart below.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3, w4]
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w5 = 0
    ASSERT w6 = 1
    ASSERT w7 = 3
    ASSERT w8 = 4
    INIT b0 = [w5, w6, w7, w8]
    READ w9 = b0[w0]
    ASSERT w10 = 2
    ASSERT w11 = 5
    ASSERT w12 = 6
    INIT b1 = [w6, w10, w7, w8, w11, w12]
    WRITE b1[w9] = w2
    READ w13 = b1[w8]
    READ w14 = b1[w11]
    ASSERT w3 = w13
    ASSERT w4 = w14
    ");
}

#[test]
fn predicated_constant_index_get_on_heterogeneous_array() {
    // The array-typed counterpart of `predicated_constant_index_get_on_heterogeneous_vector`,
    // pinning the behaviour the vector case should match: `is_safe_index` holds for an
    // in-bounds constant index into an array, so no bias is applied and both reads stay on the
    // slots the index resolved to.
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u1, v2: Field):
        v3 = make_array [u8 2, u8 3] : [u8; 2]
        v4 = make_array [u8 5, u8 6] : [u8; 2]
        v5 = make_array [Field 1, v3, Field 4, v4] : [(Field, [u8; 2]); 2]
        v6 = array_set v5, index v0, value v2
        enable_side_effects v1
        v7 = array_get v6, index u32 3 -> [u8; 2]
        enable_side_effects u1 1
        return v7
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3, w4]
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w5 = 0
    ASSERT w6 = 1
    ASSERT w7 = 3
    ASSERT w8 = 4
    INIT b0 = [w5, w6, w7, w8]
    READ w9 = b0[w0]
    ASSERT w10 = 2
    ASSERT w11 = 5
    ASSERT w12 = 6
    INIT b1 = [w6, w10, w7, w8, w11, w12]
    WRITE b1[w9] = w2
    READ w13 = b1[w8]
    READ w14 = b1[w11]
    ASSERT w3 = w13
    ASSERT w4 = w14
    ");
}
