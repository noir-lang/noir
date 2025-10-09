use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

// Tests ArrayLen intrinsic code-gen for Brillig
#[test]
fn brillig_array_len() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = call f1() -> u32
        constrain v1 == u32 3
        return
    }

    brillig(inline) fn foo f1 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1 = call array_len(v0) -> u32
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: []
public parameters: []
return values: []
BRILLIG CALL func: 0, inputs: [], outputs: [w0]
ASSERT w0 = 3

unconstrained func 0: foo
 0: @2 = const u32 1
 1: @1 = const u32 32837
 2: @0 = const u32 69
 3: sp[1] = const u32 0
 4: sp[2] = const u32 0
 5: @68 = calldata copy [sp[2]; sp[1]]
 6: call 12
 7: call 13
 8: @68 = sp[1]
 9: sp[2] = const u32 68
10: sp[3] = const u32 1
11: stop &[sp[2]; sp[3]]
12: return
13: call 30
14: sp[1] = const u32 10
15: sp[2] = const u32 20
16: sp[3] = const u32 30
17: sp[4] = @1
18: sp[5] = const u32 4
19: @1 = u32 add @1, sp[5]
20: sp[4] = indirect const u32 1
21: sp[5] = u32 add sp[4], @2
22: sp[6] = sp[5]
23: store sp[1] at sp[6]
24: sp[6] = u32 add sp[6], @2
25: store sp[2] at sp[6]
26: sp[6] = u32 add sp[6], @2
27: store sp[3] at sp[6]
28: sp[1] = const u32 3
29: return
30: @4 = const u32 30789
31: @3 = u32 lt @0, @4
32: jump if @3 to 35
33: @1 = indirect const u64 15764276373176857197
34: trap &[@1; @2]
35: return
");
}

// Tests AsSlice intrinsic code-gen for Brillig.
#[test]
fn brillig_as_slice() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = call f1() -> u32
        constrain v1 == u32 3
        return
    }

    brillig(inline) fn foo f1 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_slice(v0) -> (u32, [u32])
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [], outputs: [w0]
    ASSERT w0 = 3

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32837
     2: @0 = const u32 69
     3: sp[1] = const u32 0
     4: sp[2] = const u32 0
     5: @68 = calldata copy [sp[2]; sp[1]]
     6: call 12
     7: call 13
     8: @68 = sp[1]
     9: sp[2] = const u32 68
    10: sp[3] = const u32 1
    11: stop &[sp[2]; sp[3]]
    12: return
    13: call 47
    14: sp[1] = const u32 10
    15: sp[2] = const u32 20
    16: sp[3] = const u32 30
    17: sp[4] = @1
    18: sp[5] = const u32 4
    19: @1 = u32 add @1, sp[5]
    20: sp[4] = indirect const u32 1
    21: sp[5] = u32 add sp[4], @2
    22: sp[6] = sp[5]
    23: store sp[1] at sp[6]
    24: sp[6] = u32 add sp[6], @2
    25: store sp[2] at sp[6]
    26: sp[6] = u32 add sp[6], @2
    27: store sp[3] at sp[6]
    28: sp[3] = const u32 3
    29: sp[1] = u32 div sp[3], @2
    30: sp[6] = const u32 3
    31: sp[5] = u32 add sp[3], sp[6]
    32: sp[2] = @1
    33: @1 = u32 add @1, sp[5]
    34: sp[2] = indirect const u32 1
    35: sp[5] = u32 add sp[2], @2
    36: store sp[3] at sp[5]
    37: sp[5] = u32 add sp[5], @2
    38: store sp[3] at sp[5]
    39: sp[6] = const u32 3
    40: sp[5] = u32 add sp[2], sp[6]
    41: sp[6] = u32 add sp[4], @2
    42: @3 = sp[6]
    43: @4 = sp[5]
    44: @5 = sp[3]
    45: call 53
    46: return
    47: @4 = const u32 30789
    48: @3 = u32 lt @0, @4
    49: jump if @3 to 52
    50: @1 = indirect const u64 15764276373176857197
    51: trap &[@1; @2]
    52: return
    53: @7 = u32 add @3, @5
    54: @8 = @3
    55: @9 = @4
    56: @10 = u32 eq @8, @7
    57: jump if @10 to 63
    58: @6 = load @8
    59: store @6 at @9
    60: @8 = u32 add @8, @2
    61: @9 = u32 add @9, @2
    62: jump to 56
    63: return
    ");
}

// Tests ToBits intrinsic code-gen for Brillig.
#[test]
fn brillig_to_bits() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        v2 = call f1(v0) -> [u1; 8]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field):
        v1 = call to_le_bits(v0) -> [u1; 8]
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [w0], outputs: [[w1, w2, w3, w4, w5, w6, w7, w8]]
    BLACKBOX::RANGE input: w1, bits: 1
    BLACKBOX::RANGE input: w2, bits: 1
    BLACKBOX::RANGE input: w3, bits: 1
    BLACKBOX::RANGE input: w4, bits: 1
    BLACKBOX::RANGE input: w5, bits: 1
    BLACKBOX::RANGE input: w6, bits: 1
    BLACKBOX::RANGE input: w7, bits: 1
    BLACKBOX::RANGE input: w8, bits: 1

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32845
     2: @0 = const u32 77
     3: sp[2] = const u32 1
     4: sp[3] = const u32 0
     5: @68 = calldata copy [sp[3]; sp[2]]
     6: sp[1] = @68
     7: call 19
     8: call 20
     9: sp[2] = u32 add sp[1], @2
    10: sp[3] = const u32 69
    11: sp[4] = const u32 8
    12: @3 = sp[2]
    13: @4 = sp[3]
    14: @5 = sp[4]
    15: call 36
    16: sp[2] = const u32 69
    17: sp[3] = const u32 8
    18: stop &[sp[2]; sp[3]]
    19: return
    20: call 47
    21: sp[3] = const u32 2
    22: sp[4] = const bool 1
    23: sp[2] = @1
    24: sp[5] = const u32 9
    25: @1 = u32 add @1, sp[5]
    26: sp[2] = indirect const u32 1
    27: sp[5] = u32 add sp[2], @2
    28: sp[6] = const u32 8
    29: to_radix(input: sp[1], radix: sp[3], num_limbs: sp[6], output_pointer: sp[5], output_bits: sp[4])
    30: sp[7] = const u32 8
    31: @3 = sp[5]
    32: @4 = sp[7]
    33: call 53
    34: sp[1] = sp[2]
    35: return
    36: @7 = u32 add @3, @5
    37: @8 = @3
    38: @9 = @4
    39: @10 = u32 eq @8, @7
    40: jump if @10 to 46
    41: @6 = load @8
    42: store @6 at @9
    43: @8 = u32 add @8, @2
    44: @9 = u32 add @9, @2
    45: jump to 39
    46: return
    47: @4 = const u32 30797
    48: @3 = u32 lt @0, @4
    49: jump if @3 to 52
    50: @1 = indirect const u64 15764276373176857197
    51: trap &[@1; @2]
    52: return
    53: @6 = const u32 2
    54: @5 = u32 div @4, @6
    55: @8 = @4
    56: @9 = const u32 0
    57: @10 = u32 lt_eq @5, @9
    58: jump if @10 to 70
    59: @8 = u32 sub @8, @2
    60: @11 = u32 add @3, @9
    61: @6 = load @11
    62: @11 = u32 add @3, @8
    63: @7 = load @11
    64: @11 = u32 add @3, @9
    65: store @7 at @11
    66: @11 = u32 add @3, @8
    67: store @6 at @11
    68: @9 = u32 add @9, @2
    69: jump to 57
    70: return
    ");
}

// Tests ToRadix intrinsic code-gen for Brillig.
#[test]
fn brillig_to_radix() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: u32):
        v3 = call f1(v0, v1) -> [u8; 8]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: u32):
        v2 = call to_le_radix(v0, v1) -> [u8; 8]
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w1, bits: 32
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [[w2, w3, w4, w5, w6, w7, w8, w9]]
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32846
     2: @0 = const u32 78
     3: sp[3] = const u32 2
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: @69 = cast @69 to u32
     7: sp[1] = @68
     8: sp[2] = @69
     9: call 21
    10: call 22
    11: sp[2] = u32 add sp[1], @2
    12: sp[3] = const u32 70
    13: sp[4] = const u32 8
    14: @3 = sp[2]
    15: @4 = sp[3]
    16: @5 = sp[4]
    17: call 37
    18: sp[2] = const u32 70
    19: sp[3] = const u32 8
    20: stop &[sp[2]; sp[3]]
    21: return
    22: call 48
    23: sp[4] = const bool 0
    24: sp[3] = @1
    25: sp[5] = const u32 9
    26: @1 = u32 add @1, sp[5]
    27: sp[3] = indirect const u32 1
    28: sp[5] = u32 add sp[3], @2
    29: sp[6] = const u32 8
    30: to_radix(input: sp[1], radix: sp[2], num_limbs: sp[6], output_pointer: sp[5], output_bits: sp[4])
    31: sp[7] = const u32 8
    32: @3 = sp[5]
    33: @4 = sp[7]
    34: call 54
    35: sp[1] = sp[3]
    36: return
    37: @7 = u32 add @3, @5
    38: @8 = @3
    39: @9 = @4
    40: @10 = u32 eq @8, @7
    41: jump if @10 to 47
    42: @6 = load @8
    43: store @6 at @9
    44: @8 = u32 add @8, @2
    45: @9 = u32 add @9, @2
    46: jump to 40
    47: return
    48: @4 = const u32 30798
    49: @3 = u32 lt @0, @4
    50: jump if @3 to 53
    51: @1 = indirect const u64 15764276373176857197
    52: trap &[@1; @2]
    53: return
    54: @6 = const u32 2
    55: @5 = u32 div @4, @6
    56: @8 = @4
    57: @9 = const u32 0
    58: @10 = u32 lt_eq @5, @9
    59: jump if @10 to 71
    60: @8 = u32 sub @8, @2
    61: @11 = u32 add @3, @9
    62: @6 = load @11
    63: @11 = u32 add @3, @8
    64: @7 = load @11
    65: @11 = u32 add @3, @9
    66: store @7 at @11
    67: @11 = u32 add @3, @8
    68: store @6 at @11
    69: @9 = u32 add @9, @2
    70: jump to 58
    71: return
    ");
}

// Tests FieldLessThan intrinsic code-gen for Brillig.
#[test]
fn brillig_field_less_than() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v3 = call f1(v0, v1) -> u1
        constrain v3 == u1 1
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = call field_less_than(v0, v1) -> u1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w2]
    ASSERT w2 = 1

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
    15: call 19
    16: sp[3] = field lt sp[1], sp[2]
    17: sp[1] = sp[3]
    18: return
    19: @4 = const u32 30791
    20: @3 = u32 lt @0, @4
    21: jump if @3 to 24
    22: @1 = indirect const u64 15764276373176857197
    23: trap &[@1; @2]
    24: return
    ");
}

// Tests ArrayRefCount intrinsic code-gen for Brillig.
#[test]
fn brillig_array_ref_count() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = call f1() -> u32
        return
    }

    brillig(inline) fn foo f1 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1 = call array_refcount(v0) -> u32
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [], outputs: [w0]
    BLACKBOX::RANGE input: w0, bits: 32

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32837
     2: @0 = const u32 69
     3: sp[1] = const u32 0
     4: sp[2] = const u32 0
     5: @68 = calldata copy [sp[2]; sp[1]]
     6: call 12
     7: call 13
     8: @68 = sp[1]
     9: sp[2] = const u32 68
    10: sp[3] = const u32 1
    11: stop &[sp[2]; sp[3]]
    12: return
    13: call 30
    14: sp[1] = const u32 10
    15: sp[2] = const u32 20
    16: sp[3] = const u32 30
    17: sp[4] = @1
    18: sp[5] = const u32 4
    19: @1 = u32 add @1, sp[5]
    20: sp[4] = indirect const u32 1
    21: sp[5] = u32 add sp[4], @2
    22: sp[6] = sp[5]
    23: store sp[1] at sp[6]
    24: sp[6] = u32 add sp[6], @2
    25: store sp[2] at sp[6]
    26: sp[6] = u32 add sp[6], @2
    27: store sp[3] at sp[6]
    28: sp[1] = load sp[4]
    29: return
    30: @4 = const u32 30789
    31: @3 = u32 lt @0, @4
    32: jump if @3 to 35
    33: @1 = indirect const u64 15764276373176857197
    34: trap &[@1; @2]
    35: return
    ");
}

// Tests SliceRefCount intrinsic code-gen for Brillig.
#[test]
fn brillig_slice_ref_count() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v3 = call f1() -> u32
        return
    }

    brillig(inline) fn foo f1 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_slice(v0) -> (u32, [u32])
        v3 = call slice_refcount(v1, v2) -> u32
        return v3
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [], outputs: [w0]
    BLACKBOX::RANGE input: w0, bits: 32

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32837
     2: @0 = const u32 69
     3: sp[1] = const u32 0
     4: sp[2] = const u32 0
     5: @68 = calldata copy [sp[2]; sp[1]]
     6: call 12
     7: call 13
     8: @68 = sp[1]
     9: sp[2] = const u32 68
    10: sp[3] = const u32 1
    11: stop &[sp[2]; sp[3]]
    12: return
    13: call 49
    14: sp[1] = const u32 10
    15: sp[2] = const u32 20
    16: sp[3] = const u32 30
    17: sp[4] = @1
    18: sp[5] = const u32 4
    19: @1 = u32 add @1, sp[5]
    20: sp[4] = indirect const u32 1
    21: sp[5] = u32 add sp[4], @2
    22: sp[6] = sp[5]
    23: store sp[1] at sp[6]
    24: sp[6] = u32 add sp[6], @2
    25: store sp[2] at sp[6]
    26: sp[6] = u32 add sp[6], @2
    27: store sp[3] at sp[6]
    28: sp[3] = const u32 3
    29: sp[1] = u32 div sp[3], @2
    30: sp[6] = const u32 3
    31: sp[5] = u32 add sp[3], sp[6]
    32: sp[2] = @1
    33: @1 = u32 add @1, sp[5]
    34: sp[2] = indirect const u32 1
    35: sp[5] = u32 add sp[2], @2
    36: store sp[3] at sp[5]
    37: sp[5] = u32 add sp[5], @2
    38: store sp[3] at sp[5]
    39: sp[6] = const u32 3
    40: sp[5] = u32 add sp[2], sp[6]
    41: sp[6] = u32 add sp[4], @2
    42: @3 = sp[6]
    43: @4 = sp[5]
    44: @5 = sp[3]
    45: call 55
    46: sp[3] = load sp[2]
    47: sp[1] = sp[3]
    48: return
    49: @4 = const u32 30789
    50: @3 = u32 lt @0, @4
    51: jump if @3 to 54
    52: @1 = indirect const u64 15764276373176857197
    53: trap &[@1; @2]
    54: return
    55: @7 = u32 add @3, @5
    56: @8 = @3
    57: @9 = @4
    58: @10 = u32 eq @8, @7
    59: jump if @10 to 65
    60: @6 = load @8
    61: store @6 at @9
    62: @8 = u32 add @8, @2
    63: @9 = u32 add @9, @2
    64: jump to 58
    65: return
    ");
}
