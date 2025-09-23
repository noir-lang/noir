use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn does_not_generate_memory_blocks_without_dynamic_accesses() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
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
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w1
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ 2 ], [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]]], outputs: []
    EXPR [ (1, w0) 0 ]

    unconstrained func 0
     0: @2 = const u32 1
     1: @1 = const u32 32839
     2: @0 = const u32 3
     3: sp[3] = const u32 3
     4: sp[4] = const u32 0
     5: @32836 = calldata copy [sp[4]; sp[3]]
     6: @32836 = cast @32836 to u32
     7: sp[1] = @32836
     8: sp[2] = const u32 32837
     9: sp[4] = const u32 2
    10: sp[6] = const u32 3
    11: sp[5] = u32 add sp[4], sp[6]
    12: sp[3] = @1
    13: @1 = u32 add @1, sp[5]
    14: sp[3] = indirect const u32 1
    15: sp[5] = u32 add sp[3], @2
    16: store sp[4] at sp[5]
    17: sp[5] = u32 add sp[5], @2
    18: store sp[4] at sp[5]
    19: sp[6] = const u32 3
    20: sp[5] = u32 add sp[3], sp[6]
    21: @32771 = sp[2]
    22: @32772 = sp[5]
    23: @32773 = sp[4]
    24: call 31
    25: sp[2] = sp[3]
    26: call 42
    27: call 43
    28: sp[1] = const u32 32839
    29: sp[2] = const u32 0
    30: stop &[sp[1]; sp[2]]
    31: @32775 = u32 add @32771, @32773
    32: @32776 = @32771
    33: @32777 = @32772
    34: @32778 = u32 eq @32776, @32775
    35: jump if @32778 to 41
    36: @32774 = load @32776
    37: store @32774 at @32777
    38: @32776 = u32 add @32776, @2
    39: @32777 = u32 add @32777, @2
    40: jump to 34
    41: return
    42: return
    43: call 45
    44: return
    45: @32772 = const u32 30720
    46: @32771 = u32 lt @0, @32772
    47: jump if @32771 to 50
    48: @1 = indirect const u64 15764276373176857197
    49: trap &[@1; @2]
    50: return
    ");
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
    current witness: w3
    private parameters: []
    public parameters: []
    return values: []
    EXPR [ (-1, w0) 0 ]
    EXPR [ (-1, w1) 1 ]
    INIT (id: 0, len: 2, witnesses: [w0, w1])
    EXPR [ (-1, w2) 5 ]
    MEM (id: 0, read at: EXPR [ (1, w2) 0 ], value: EXPR [ (1, w3) 0 ]) 
    EXPR [ (1, w3) 0 ]
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
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w0
    private parameters: []
    public parameters: []
    return values: []
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
    current witness: w4
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: []
    INIT (id: 0, len: 3, witnesses: [w0, w1, w2])
    MEM (id: 0, read at: EXPR [ (1, w3) 0 ], value: EXPR [ (1, w4) 0 ]) 
    EXPR [ (1, w4) -10 ]
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
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w13
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: [w4, w5, w6]
    INIT (id: 1, len: 3, witnesses: [w0, w1, w2])
    EXPR [ (-1, w7) 10 ]
    MEM (id: 1, write EXPR [ (1, w7) 0 ] at: EXPR [ (1, w3) 0 ]) 
    EXPR [ (-1, w8) 0 ]
    MEM (id: 1, read at: EXPR [ (1, w8) 0 ], value: EXPR [ (1, w9) 0 ]) 
    EXPR [ (-1, w10) 1 ]
    MEM (id: 1, read at: EXPR [ (1, w10) 0 ], value: EXPR [ (1, w11) 0 ]) 
    EXPR [ (-1, w12) 2 ]
    MEM (id: 1, read at: EXPR [ (1, w12) 0 ], value: EXPR [ (1, w13) 0 ]) 
    EXPR [ (1, w4) (-1, w9) 0 ]
    EXPR [ (1, w5) (-1, w11) 0 ]
    EXPR [ (1, w6) (-1, w13) 0 ]
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
    // We can see that before the read we have `EXPR [ (1, w3, w4) (-1, w5) 0 ]`
    // As the index is zero this is a simplified version of `index*predicate + (1-predicate)*offset`
    // w5 is then used as the index which we use to read from the memory block
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w6
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    INIT (id: 0, len: 3, witnesses: [w0, w1, w2])
    BLACKBOX::RANGE [w3]:32 bits []
    BLACKBOX::RANGE [w4]:1 bits []
    EXPR [ (1, w3, w4) (-1, w5) 0 ]
    MEM (id: 0, read at: EXPR [ (1, w5) 0 ], value: EXPR [ (1, w6) 0 ]) 
    EXPR [ (1, w6) -10 ]
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

    // Similar to the `generates_predicated_index_for_dynamic_read` test we can
    // see how `EXPR [ (1, w3, w4) (-1, w8) 0 ]` forms our predicated index.
    // However, now we also have extra logic for generating a dummy value.
    // The original value we want to write is `Field 10` and our predicate is `w4`.
    // We read the value at the predicated index into `w9`. This is our dummy value.
    // We can then see how we form our new store value with:
    // `EXPR [ (-1, w4, w9) (10, w4) (1, w9) (-1, w10) 0 ]` -> (predicate*value + (1-predicate)*dummy)
    // `(10, w4)` -> predicate*value
    // `(-1, w4, w9)` -> (-predicate * dummy)
    // `(1, w9)` -> dummy
    // As expected, we then store `w10` at the predicated index `w8`.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w16
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT (id: 0, len: 3, witnesses: [w0, w1, w2])
    BLACKBOX::RANGE [w3]:32 bits []
    BLACKBOX::RANGE [w4]:1 bits []
    EXPR [ (1, w3, w4) (-1, w8) 0 ]
    MEM (id: 0, read at: EXPR [ (1, w8) 0 ], value: EXPR [ (1, w9) 0 ]) 
    INIT (id: 1, len: 3, witnesses: [w0, w1, w2])
    EXPR [ (-1, w4, w9) (10, w4) (1, w9) (-1, w10) 0 ]
    MEM (id: 1, write EXPR [ (1, w10) 0 ] at: EXPR [ (1, w8) 0 ]) 
    EXPR [ (-1, w11) 0 ]
    MEM (id: 1, read at: EXPR [ (1, w11) 0 ], value: EXPR [ (1, w12) 0 ]) 
    EXPR [ (-1, w13) 1 ]
    MEM (id: 1, read at: EXPR [ (1, w13) 0 ], value: EXPR [ (1, w14) 0 ]) 
    EXPR [ (-1, w15) 2 ]
    MEM (id: 1, read at: EXPR [ (1, w15) 0 ], value: EXPR [ (1, w16) 0 ]) 
    EXPR [ (1, w5) (-1, w12) 0 ]
    EXPR [ (1, w6) (-1, w14) 0 ]
    EXPR [ (1, w7) (-1, w16) 0 ]
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
    current witness: w0
    private parameters: []
    public parameters: []
    return values: []
    EXPR [ 1 ]
    ");
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
    current witness: w0
    private parameters: [w0]
    public parameters: []
    return values: []
    EXPR [ (1, w0) 0 ]
    ");
}
