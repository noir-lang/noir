use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn slice_push_back() {
    // This SSA would never be generated as we are writing to a slice without a preceding OOB check.
    // We forego the OOB check here for the succinctness of the test.
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32):
        v3 = make_array [Field 2, Field 3] : [Field]
        // Mutate it at index v0 (to make it non-constant in the circuit)
        v5 = array_set v3, index v0, value Field 4
        v8, v9 = call slice_push_back(u32 2, v5, Field 10) -> (u32, [Field])
        constrain v8 == v1
        // Mutate the new slice to make the result block observable
        v11 = array_set v9, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Note that w9 is now at the end of memory block 3 and that w1 has been asserted to equal 3
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    EXPR w2 = 2
    EXPR w3 = 3
    INIT id: 1, len: 2, witnesses: [w2, w3]
    EXPR w4 = 4
    MEM id: 1, write: w4 at: w0
    EXPR w5 = 0
    MEM id: 1, read at: w5, value: w6
    EXPR w7 = 1
    MEM id: 1, read at: w7, value: w8
    EXPR w9 = 10
    EXPR w1 = 3
    INIT id: 3, len: 3, witnesses: [w6, w8, w9]
    EXPR w10 = 20
    MEM id: 3, write: w10 at: w0
    ");
}

#[test]
fn slice_push_front() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32):
        v3 = make_array [Field 2, Field 3] : [Field]
        // Mutate it at index v0 (to make it non-constant in the circuit)
        v5 = array_set v3, index v0, value Field 4
        v8, v9 = call slice_push_front(u32 2, v5, Field 10) -> (u32, [Field])
        constrain v8 == v1
        // Mutate the new slice to make the result block observable
        v11 = array_set v9, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Note that w9 is now in the front of memory block 3 and that w1 has been asserted to equal 3
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    EXPR w2 = 2
    EXPR w3 = 3
    INIT id: 1, len: 2, witnesses: [w2, w3]
    EXPR w4 = 4
    MEM id: 1, write: w4 at: w0
    EXPR w5 = 0
    MEM id: 1, read at: w5, value: w6
    EXPR w7 = 1
    MEM id: 1, read at: w7, value: w8
    EXPR w9 = 10
    EXPR w1 = 3
    INIT id: 3, len: 3, witnesses: [w9, w6, w8]
    EXPR w10 = 20
    MEM id: 3, write: w10 at: w0
    ");
}

#[test]
fn slice_pop_back() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32):
        v3 = make_array [Field 2, Field 3] : [Field]
        // Mutate it at index v0 (to make it non-constant in the circuit)
        v5 = array_set v3, index v0, value Field 4
        v8, v9, v10 = call slice_pop_back(u32 2, v5) -> (u32, [Field], Field)
        constrain v8 == v1
        constrain v10 == Field 3
        // Mutate the new slice to make the result block observable
        v12 = array_set v9, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // As you can see we read the entire slice in order (memory block 1) and that w1 has been asserted to equal 1
    // In practice, when writing to the slice we would assert that the index is less than w1
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    EXPR w2 = 2
    EXPR w3 = 3
    INIT id: 1, len: 2, witnesses: [w2, w3]
    EXPR w4 = 4
    MEM id: 1, write: w4 at: w0
    EXPR w5 = 1
    MEM id: 1, read at: w5, value: w6
    EXPR w7 = 0
    MEM id: 1, read at: w7, value: w8
    MEM id: 1, read at: w5, value: w9
    EXPR w1 = 1
    EXPR w6 = 3
    INIT id: 3, len: 2, witnesses: [w8, w9]
    EXPR w10 = 20
    MEM id: 3, write: w10 at: w0
    ");
}

#[test]
fn slice_pop_front() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32):
        v3 = make_array [Field 2, Field 3] : [Field]
        v5 = array_set v3, index v0, value Field 4
        v8, v9, v10 = call slice_pop_front(u32 2, v5) -> (Field, u32, [Field])
        constrain v8 == Field 2
        constrain v9 == v1
        v12 = array_set v10, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // As you can see we read the entire slice in order (memory block 1) and that w1 has been asserted to equal 1
    // In practice, when writing to the slice we would assert that the index is less than w1
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    EXPR w2 = 2
    EXPR w3 = 3
    INIT id: 1, len: 2, witnesses: [w2, w3]
    EXPR w4 = 4
    MEM id: 1, write: w4 at: w0
    EXPR w5 = 0
    MEM id: 1, read at: w5, value: w6
    EXPR w7 = 1
    MEM id: 1, read at: w7, value: w8
    MEM id: 1, read at: w5, value: w9
    EXPR w9 = 2
    EXPR w1 = 1
    INIT id: 3, len: 1, witnesses: [w8]
    EXPR w10 = 20
    MEM id: 3, write: w10 at: w0
    ");
}

// TODO(https://github.com/noir-lang/noir/issues/10015)
#[test]
fn slice_insert() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32):
        v4 = make_array [Field 2, Field 3, Field 5] : [Field]
        v6 = array_set v4, index v0, value Field 4
        v10, v11 = call slice_insert(u32 3, v6, v1, Field 10) -> (u32, [Field])
        constrain v10 == v1
        v13 = array_set v11, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Insert does comparisons on every index for the value that should be written into the resulting slice
    // You can see how w1 is asserted to equal 4
    // Memory block 1 is our original slice
    // Memory block 2 is our temporary slice while shifting. You can see its contents are all w6 (which is equal to 0).
    // TODO(https://github.com/noir-lang/noir/issues/10015): Memory block 3 is our final slice which remains three elements. This is incorrect.
    // The Brillig calls are to our stdlib quotient directive
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w34
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [w1]:32 bits []
    EXPR w2 = 2
    EXPR w3 = 3
    EXPR w4 = 5
    INIT id: 1, len: 3, witnesses: [w2, w3, w4]
    EXPR w5 = 4
    MEM id: 1, write: w5 at: w0
    EXPR w6 = 0
    INIT id: 2, len: 3, witnesses: [w6, w6, w6]
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w7, w8]
    BLACKBOX::RANGE [w7]:1 bits []
    BLACKBOX::RANGE [w8]:64 bits []
    EXPR w8 = -w1 - 18446744073709551616*w7 + 18446744073709551616
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551615, 18446744073709551616], outputs: [w9, w10]
    BLACKBOX::RANGE [w9]:1 bits []
    BLACKBOX::RANGE [w10]:64 bits []
    EXPR w10 = -w1 - 18446744073709551616*w9 + 18446744073709551615
    MEM id: 1, read at: w6, value: w11
    EXPR w12 = w7*w9 - w7 + 1
    EXPR w13 = -10*w7*w9 + w11*w12 + 10*w7
    MEM id: 2, write: w13 at: w6
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w14, w15]
    BLACKBOX::RANGE [w14]:1 bits []
    BLACKBOX::RANGE [w15]:64 bits []
    EXPR w15 = -w1 - 18446744073709551616*w14 + 18446744073709551617
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w16, w17]
    BLACKBOX::RANGE [w16]:1 bits []
    BLACKBOX::RANGE [w17]:64 bits []
    EXPR w17 = -w1 - 18446744073709551616*w16 + 18446744073709551616
    EXPR w18 = -w14 + 1
    MEM id: 1, read at: w18, value: w19
    EXPR w20 = w14*w16 - w14 + 1
    EXPR w21 = 1
    EXPR w22 = -10*w14*w16 + w19*w20 + 10*w14
    MEM id: 2, write: w22 at: w21
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551618, 18446744073709551616], outputs: [w23, w24]
    BLACKBOX::RANGE [w23]:1 bits []
    BLACKBOX::RANGE [w24]:64 bits []
    EXPR w24 = -w1 - 18446744073709551616*w23 + 18446744073709551618
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w25, w26]
    BLACKBOX::RANGE [w25]:1 bits []
    BLACKBOX::RANGE [w26]:64 bits []
    EXPR w26 = -w1 - 18446744073709551616*w25 + 18446744073709551617
    EXPR w27 = -w23 + 2
    MEM id: 1, read at: w27, value: w28
    EXPR w29 = w23*w25 - w23 + 1
    EXPR w30 = -10*w23*w25 + w28*w29 + 10*w23
    MEM id: 2, write: w30 at: w2
    EXPR w1 = 4
    MEM id: 2, read at: w6, value: w31
    MEM id: 2, read at: w21, value: w32
    MEM id: 2, read at: w2, value: w33
    INIT id: 3, len: 3, witnesses: [w31, w32, w33]
    EXPR w34 = 20
    MEM id: 3, write: w34 at: w0
    
    unconstrained func 0
    0: @10 = const u32 2
    1: @11 = const u32 0
    2: @0 = calldata copy [@11; @10]
    3: @2 = field int_div @0, @1
    4: @1 = field mul @2, @1
    5: @1 = field sub @0, @1
    6: @0 = @2
    7: stop &[@11; @10]
    ");
}

#[test]
fn slice_remove() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u32, v2: Field):
        v6 = make_array [Field 2, Field 3, Field 5] : [Field]
        v8 = array_set v6, index v0, value Field 4
        v11, v12, v13 = call slice_remove(u32 3, v8, v1) -> (u32, [Field], Field)
        constrain v11 == v1
        constrain v13 == v2
        v15 = array_set v12, index v0, value Field 20
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Remove does comparisons on every index for the value that should be written into the resulting slice
    // You can see how w1 is asserted to equal 4
    // Memory block 1 is our original slice
    // Memory block 2 is our temporary slice while shifting.
    // Memory block 3 is our final slice which remains three elements.
    // The Brillig calls are to our stdlib quotient directive
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w24
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR w3 = 2
    EXPR w4 = 3
    EXPR w5 = 5
    INIT id: 1, len: 3, witnesses: [w3, w4, w5]
    EXPR w6 = 4
    MEM id: 1, write: w6 at: w0
    EXPR w7 = 0
    MEM id: 1, read at: w7, value: w8
    EXPR w9 = 1
    MEM id: 1, read at: w9, value: w10
    MEM id: 1, read at: w3, value: w11
    MEM id: 1, read at: w1, value: w12
    INIT id: 2, len: 3, witnesses: [w8, w10, w11]
    MEM id: 1, read at: w9, value: w13
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w14, w15]
    BLACKBOX::RANGE [w14]:1 bits []
    BLACKBOX::RANGE [w15]:64 bits []
    EXPR w15 = -w1 - 18446744073709551616*w14 + 18446744073709551616
    EXPR w16 = w13*w14 - w8*w14 + w8
    MEM id: 2, write: w16 at: w7
    MEM id: 1, read at: w3, value: w17
    BRILLIG CALL func 0: inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w18, w19]
    BLACKBOX::RANGE [w18]:1 bits []
    BLACKBOX::RANGE [w19]:64 bits []
    EXPR w19 = -w1 - 18446744073709551616*w18 + 18446744073709551617
    EXPR w20 = w17*w18 - w10*w18 + w10
    MEM id: 2, write: w20 at: w9
    EXPR w1 = 2
    EXPR w12 = w2
    MEM id: 2, read at: w7, value: w21
    MEM id: 2, read at: w9, value: w22
    MEM id: 2, read at: w3, value: w23
    INIT id: 3, len: 3, witnesses: [w21, w22, w23]
    EXPR w24 = 20
    MEM id: 3, write: w24 at: w0
    
    unconstrained func 0
    0: @10 = const u32 2
    1: @11 = const u32 0
    2: @0 = calldata copy [@11; @10]
    3: @2 = field int_div @0, @1
    4: @1 = field mul @2, @1
    5: @1 = field sub @0, @1
    6: @0 = @2
    7: stop &[@11; @10]
    ");
}

#[test]
fn slice_push_back_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_push_back(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_push_back(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_push_front_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_push_front(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_push_front(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_pop_back_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_pop_back(u32 1, v7) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_pop_back(u32 1, v7) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_pop_front_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_pop_front(u32 1, v7) -> (Field, [Field; 2], u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_pop_front(u32 1, v7) -> (Field, [Field; 2], u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_insert_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_insert(u32 1, v7, u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_insert(u32 1, v7, u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_ne!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_remove_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_remove(u32 1, v7, u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_remove(u32 1, v7, u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(src_no_side_effects);
    assert_ne!(program_side_effects, program_no_side_effects);
}
