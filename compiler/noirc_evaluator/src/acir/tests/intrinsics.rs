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
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    ASSERT w2 = 2
    ASSERT w3 = 3
    INIT b1 = [w2, w3]
    ASSERT w4 = 4
    WRITE b1[w0] = w4
    ASSERT w5 = 0
    READ w6 = b1[w5]
    ASSERT w7 = 1
    READ w8 = b1[w7]
    ASSERT w9 = 10
    ASSERT w1 = 3
    INIT b3 = [w6, w8, w9]
    ASSERT w10 = 20
    WRITE b3[w0] = w10
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
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    ASSERT w2 = 2
    ASSERT w3 = 3
    INIT b1 = [w2, w3]
    ASSERT w4 = 4
    WRITE b1[w0] = w4
    ASSERT w5 = 0
    READ w6 = b1[w5]
    ASSERT w7 = 1
    READ w8 = b1[w7]
    ASSERT w9 = 10
    ASSERT w1 = 3
    INIT b3 = [w9, w6, w8]
    ASSERT w10 = 20
    WRITE b3[w0] = w10
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
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    ASSERT w2 = 2
    ASSERT w3 = 3
    INIT b1 = [w2, w3]
    ASSERT w4 = 4
    WRITE b1[w0] = w4
    ASSERT w5 = 1
    READ w6 = b1[w5]
    ASSERT w7 = 0
    READ w8 = b1[w7]
    READ w9 = b1[w5]
    ASSERT w1 = 1
    ASSERT w6 = 3
    INIT b3 = [w8, w9]
    ASSERT w10 = 20
    WRITE b3[w0] = w10
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
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    ASSERT w2 = 2
    ASSERT w3 = 3
    INIT b1 = [w2, w3]
    ASSERT w4 = 4
    WRITE b1[w0] = w4
    ASSERT w5 = 0
    READ w6 = b1[w5]
    ASSERT w7 = 1
    READ w8 = b1[w7]
    READ w9 = b1[w5]
    ASSERT w9 = 2
    ASSERT w1 = 1
    INIT b3 = [w8]
    ASSERT w10 = 20
    WRITE b3[w0] = w10
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
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w1, bits: 32
    ASSERT w2 = 2
    ASSERT w3 = 3
    ASSERT w4 = 5
    INIT b1 = [w2, w3, w4]
    ASSERT w5 = 4
    WRITE b1[w0] = w5
    ASSERT w6 = 0
    INIT b2 = [w6, w6, w6]
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w7, w8]
    BLACKBOX::RANGE input: w7, bits: 1
    BLACKBOX::RANGE input: w8, bits: 64
    ASSERT w8 = -w1 - 18446744073709551616*w7 + 18446744073709551616
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551615, 18446744073709551616], outputs: [w9, w10]
    BLACKBOX::RANGE input: w9, bits: 1
    BLACKBOX::RANGE input: w10, bits: 64
    ASSERT w10 = -w1 - 18446744073709551616*w9 + 18446744073709551615
    READ w11 = b1[w6]
    ASSERT w12 = w7*w9 - w7 + 1
    ASSERT w13 = -10*w7*w9 + w11*w12 + 10*w7
    WRITE b2[w6] = w13
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w14, w15]
    BLACKBOX::RANGE input: w14, bits: 1
    BLACKBOX::RANGE input: w15, bits: 64
    ASSERT w15 = -w1 - 18446744073709551616*w14 + 18446744073709551617
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w16, w17]
    BLACKBOX::RANGE input: w16, bits: 1
    BLACKBOX::RANGE input: w17, bits: 64
    ASSERT w17 = -w1 - 18446744073709551616*w16 + 18446744073709551616
    ASSERT w18 = -w14 + 1
    READ w19 = b1[w18]
    ASSERT w20 = w14*w16 - w14 + 1
    ASSERT w21 = 1
    ASSERT w22 = -10*w14*w16 + w19*w20 + 10*w14
    WRITE b2[w21] = w22
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551618, 18446744073709551616], outputs: [w23, w24]
    BLACKBOX::RANGE input: w23, bits: 1
    BLACKBOX::RANGE input: w24, bits: 64
    ASSERT w24 = -w1 - 18446744073709551616*w23 + 18446744073709551618
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w25, w26]
    BLACKBOX::RANGE input: w25, bits: 1
    BLACKBOX::RANGE input: w26, bits: 64
    ASSERT w26 = -w1 - 18446744073709551616*w25 + 18446744073709551617
    ASSERT w27 = -w23 + 2
    READ w28 = b1[w27]
    ASSERT w29 = w23*w25 - w23 + 1
    ASSERT w30 = -10*w23*w25 + w28*w29 + 10*w23
    WRITE b2[w2] = w30
    ASSERT w1 = 4
    READ w31 = b2[w6]
    READ w32 = b2[w21]
    READ w33 = b2[w2]
    INIT b3 = [w31, w32, w33]
    ASSERT w34 = 20
    WRITE b3[w0] = w34

    unconstrained func 0: directive_integer_quotient
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
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    ASSERT w3 = 2
    ASSERT w4 = 3
    ASSERT w5 = 5
    INIT b1 = [w3, w4, w5]
    ASSERT w6 = 4
    WRITE b1[w0] = w6
    ASSERT w7 = 0
    READ w8 = b1[w7]
    ASSERT w9 = 1
    READ w10 = b1[w9]
    READ w11 = b1[w3]
    READ w12 = b1[w1]
    INIT b2 = [w8, w10, w11]
    READ w13 = b1[w9]
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551616, 18446744073709551616], outputs: [w14, w15]
    BLACKBOX::RANGE input: w14, bits: 1
    BLACKBOX::RANGE input: w15, bits: 64
    ASSERT w15 = -w1 - 18446744073709551616*w14 + 18446744073709551616
    ASSERT w16 = w13*w14 - w8*w14 + w8
    WRITE b2[w7] = w16
    READ w17 = b1[w3]
    BRILLIG CALL func: 0, inputs: [-w1 + 18446744073709551617, 18446744073709551616], outputs: [w18, w19]
    BLACKBOX::RANGE input: w18, bits: 1
    BLACKBOX::RANGE input: w19, bits: 64
    ASSERT w19 = -w1 - 18446744073709551616*w18 + 18446744073709551617
    ASSERT w20 = w17*w18 - w10*w18 + w10
    WRITE b2[w9] = w20
    ASSERT w1 = 2
    ASSERT w12 = w2
    READ w21 = b2[w7]
    READ w22 = b2[w9]
    READ w23 = b2[w3]
    INIT b3 = [w21, w22, w23]
    ASSERT w24 = 20
    WRITE b3[w0] = w24

    unconstrained func 0: directive_integer_quotient
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
