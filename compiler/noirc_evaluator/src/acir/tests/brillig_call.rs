use acvm::{acir::circuit::brillig::BrilligFunctionId, assert_circuit_snapshot};

use crate::acir::tests::ssa_to_acir_program_with_debug_info;

// Test that given multiple calls to the same brillig function we generate only one bytecode
// and the appropriate Brillig call opcodes are generated
#[test]
fn multiple_brillig_calls_one_bytecode() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v4 = call f1(v0, v1) -> Field
        v5 = call f1(v0, v1) -> Field
        v6 = call f1(v0, v1) -> Field
        v7 = call f2(v0, v1) -> Field
        v8 = call f1(v0, v1) -> Field
        v9 = call f2(v0, v1) -> Field
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    brillig(inline) fn foo f2 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    let main_debug = &debug[0];
    // We have two normal Brillig functions that were called multiple times.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(main_debug.brillig_locations.len(), 2);
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(0)));
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(1)));

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w2]
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w4]
    BRILLIG CALL func: 1, inputs: [w0, w1], outputs: [w5]
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w6]
    BRILLIG CALL func: 1, inputs: [w0, w1], outputs: [w7]

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32839
     2: @0 = const u32 71
     3: sp[3] = const u32 2
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: sp[1] = @68
     7: sp[2] = @69
     8: call 14
     9: call 15
    10: @70 = sp[1]
    11: sp[2] = const u32 70
    12: sp[3] = const u32 1
    13: stop &[sp[2]; sp[3]]
    14: return
    15: call 23
    16: sp[3] = field eq sp[1], sp[2]
    17: sp[2] = const bool 0
    18: sp[4] = bool eq sp[3], sp[2]
    19: jump if sp[4] to 22
    20: sp[5] = const u32 0
    21: trap &[@1; sp[5]]
    22: return
    23: @4 = const u32 30791
    24: @3 = u32 lt @0, @4
    25: jump if @3 to 28
    26: @1 = indirect const u64 15764276373176857197
    27: trap &[@1; @2]
    28: return
    unconstrained func 1: foo
     0: @2 = const u32 1
     1: @1 = const u32 32839
     2: @0 = const u32 71
     3: sp[3] = const u32 2
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: sp[1] = @68
     7: sp[2] = @69
     8: call 14
     9: call 15
    10: @70 = sp[1]
    11: sp[2] = const u32 70
    12: sp[3] = const u32 1
    13: stop &[sp[2]; sp[3]]
    14: return
    15: call 23
    16: sp[3] = field eq sp[1], sp[2]
    17: sp[2] = const bool 0
    18: sp[4] = bool eq sp[3], sp[2]
    19: jump if sp[4] to 22
    20: sp[5] = const u32 0
    21: trap &[@1; sp[5]]
    22: return
    23: @4 = const u32 30791
    24: @3 = u32 lt @0, @4
    25: jump if @3 to 28
    26: @1 = indirect const u64 15764276373176857197
    27: trap &[@1; @2]
    28: return
    ");
}

// Test that given multiple primitive operations that are represented by Brillig directives (e.g. invert/quotient),
// we will only generate one bytecode and the appropriate Brillig call opcodes are generated.
#[test]
fn multiple_brillig_stdlib_calls() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v3 = div v0, v1
        constrain v3 == v2
        v4 = div v1, v2
        constrain v4 == u32 1
        return
    }";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect two brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    assert_eq!(
        program.unconstrained_functions.len(),
        2,
        "Should only have generated two Brillig functions"
    );
    assert_eq!(
        debug[0].brillig_locations.len(),
        0,
        "Brillig stdlib functions do not have location information"
    );

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w2, bits: 32
    BRILLIG CALL func: 0, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    BRILLIG CALL func: 1, inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE input: w4, bits: 32
    BLACKBOX::RANGE input: w5, bits: 32
    ASSERT w6 = w1 - w5 - 1
    BLACKBOX::RANGE input: w6, bits: 32
    ASSERT w5 = -w1*w4 + w0
    ASSERT w4 = w2
    BRILLIG CALL func: 0, inputs: [w2], outputs: [w7]
    ASSERT 0 = w2*w7 - 1
    BRILLIG CALL func: 1, inputs: [w1, w2], outputs: [w8, w9]
    BLACKBOX::RANGE input: w9, bits: 32
    ASSERT w10 = w2 - w9 - 1
    BLACKBOX::RANGE input: w10, bits: 32
    ASSERT w9 = -w2*w8 + w1
    ASSERT w8 = 1

    unconstrained func 0: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    unconstrained func 1: directive_integer_quotient
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

// Test that given both hardcoded Brillig directives and calls to normal Brillig functions,
// we generate a single bytecode for the directives and a single bytecode for the normal Brillig calls.
#[test]
fn brillig_stdlib_calls_with_regular_brillig_call() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = div v0, v1
        constrain v4 == v2
        v5 = call f1(v0, v1) -> u32
        v6 = call f1(v0, v1) -> u32
        v7 = div v1, v2
        constrain v7 == u32 1
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(
        program.unconstrained_functions.len(),
        3,
        "Should only have generated three Brillig functions"
    );
    // We have one normal Brillig functions that was called twice.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(debug[0].brillig_locations.len(), 1);
    assert!(debug[0].brillig_locations.contains_key(&BrilligFunctionId(0)));

    // Brillig stdlib IDs are expected to always come at the end of the Brillig functions list.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w2, bits: 32
    BRILLIG CALL func: 1, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    BRILLIG CALL func: 2, inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE input: w4, bits: 32
    BLACKBOX::RANGE input: w5, bits: 32
    ASSERT w6 = w1 - w5 - 1
    BLACKBOX::RANGE input: w6, bits: 32
    ASSERT w5 = -w1*w4 + w0
    ASSERT w4 = w2
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w7]
    BLACKBOX::RANGE input: w7, bits: 32
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w8]
    BLACKBOX::RANGE input: w8, bits: 32
    BRILLIG CALL func: 1, inputs: [w2], outputs: [w9]
    ASSERT 0 = w2*w9 - 1
    BRILLIG CALL func: 2, inputs: [w1, w2], outputs: [w10, w11]
    BLACKBOX::RANGE input: w11, bits: 32
    ASSERT w12 = w2 - w11 - 1
    BLACKBOX::RANGE input: w12, bits: 32
    ASSERT w11 = -w2*w10 + w1
    ASSERT w10 = 1

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32839
     2: @0 = const u32 71
     3: sp[3] = const u32 2
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: @68 = cast @68 to u32
     7: @69 = cast @69 to u32
     8: sp[1] = @68
     9: sp[2] = @69
    10: call 16
    11: call 17
    12: @70 = sp[1]
    13: sp[2] = const u32 70
    14: sp[3] = const u32 1
    15: stop &[sp[2]; sp[3]]
    16: return
    17: call 25
    18: sp[3] = u32 eq sp[1], sp[2]
    19: sp[2] = const bool 0
    20: sp[4] = bool eq sp[3], sp[2]
    21: jump if sp[4] to 24
    22: sp[5] = const u32 0
    23: trap &[@1; sp[5]]
    24: return
    25: @4 = const u32 30791
    26: @3 = u32 lt @0, @4
    27: jump if @3 to 30
    28: @1 = indirect const u64 15764276373176857197
    29: trap &[@1; @2]
    30: return
    unconstrained func 1: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    unconstrained func 2: directive_integer_quotient
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

// Test that given both normal Brillig calls, Brillig stdlib calls, and non-inlined ACIR calls, that we accurately generate ACIR.
#[test]
fn brillig_stdlib_calls_with_multiple_acir_calls() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v5 = div v0, v1
        constrain v5 == v2
        v6 = call f1(v0, v1) -> u32
        v7 = call f1(v0, v1) -> u32
        v8 = call f2(v0, v1) -> u32
        v9 = div v1, v2
        constrain v9 == u32 1
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    acir(fold) fn foo f2 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(
        program.unconstrained_functions.len(),
        3,
        "Should only have generated three Brillig functions"
    );

    let main_debug = &debug[0];
    assert_eq!(main_debug.brillig_locations.len(), 1);
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(0)));

    let foo_debug = &debug[1];
    assert_eq!(foo_debug.brillig_locations.len(), 0);

    // TODO(https://github.com/noir-lang/noir/issues/9877): Update this snapshot once the linked issue is fixed.
    // `CALL func: 2` in `func 0` is incorrect.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w2, bits: 32
    BRILLIG CALL func: 1, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    BRILLIG CALL func: 2, inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE input: w4, bits: 32
    BLACKBOX::RANGE input: w5, bits: 32
    ASSERT w6 = w1 - w5 - 1
    BLACKBOX::RANGE input: w6, bits: 32
    ASSERT w5 = -w1*w4 + w0
    ASSERT w4 = w2
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w7]
    BLACKBOX::RANGE input: w7, bits: 32
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w8]
    BLACKBOX::RANGE input: w8, bits: 32
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w9]
    BRILLIG CALL func: 1, inputs: [w2], outputs: [w10]
    ASSERT 0 = w2*w10 - 1
    BRILLIG CALL func: 2, inputs: [w1, w2], outputs: [w11, w12]
    BLACKBOX::RANGE input: w12, bits: 32
    ASSERT w13 = w2 - w12 - 1
    BLACKBOX::RANGE input: w13, bits: 32
    ASSERT w12 = -w2*w11 + w1
    ASSERT w11 = 1

    func 1
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    ASSERT w3 = w0 - w1
    BRILLIG CALL func: 1, inputs: [w3], outputs: [w4]
    ASSERT w5 = -w3*w4 + 1
    ASSERT 0 = w3*w5
    ASSERT w5 = 0
    ASSERT w2 = w0

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32839
     2: @0 = const u32 71
     3: sp[3] = const u32 2
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: @68 = cast @68 to u32
     7: @69 = cast @69 to u32
     8: sp[1] = @68
     9: sp[2] = @69
    10: call 16
    11: call 17
    12: @70 = sp[1]
    13: sp[2] = const u32 70
    14: sp[3] = const u32 1
    15: stop &[sp[2]; sp[3]]
    16: return
    17: call 25
    18: sp[3] = u32 eq sp[1], sp[2]
    19: sp[2] = const bool 0
    20: sp[4] = bool eq sp[3], sp[2]
    21: jump if sp[4] to 24
    22: sp[5] = const u32 0
    23: trap &[@1; sp[5]]
    24: return
    25: @4 = const u32 30791
    26: @3 = u32 lt @0, @4
    27: jump if @3 to 30
    28: @1 = indirect const u64 15764276373176857197
    29: trap &[@1; @2]
    30: return
    unconstrained func 1: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    unconstrained func 2: directive_integer_quotient
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
