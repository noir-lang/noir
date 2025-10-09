use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

// Tests Brillig u32 addition code-gen. It includes overflow check.
#[test]
fn brillig_add() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }
       
    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = add v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);

    assert_eq!(program.unconstrained_functions.len(), 1);

    // foo is a very simple function returning the addtion of its inputs, which is done at line 18.
    // The rest of the code is some overhead added to every brillig function for handling inputs, outputs, globals, ...
    //
    // Here is a breakdown of the code:
    // Line 1-9: The function starts with the handling of the inputs via the 'calldata copy'
    // The inputs are put in registers 1 and 2
    // Line 10: Calling the part that deals with brillig globals. In this simple example, there are no globals so
    // it just returns immediately.
    // Line 11: Calling the check_max_stack_depth_procedure (Lines 24-29), via the call 24. Returning from
    // this procedure is going to line 18 (right after the call 24 on line 17)
    // Line 18: the actual body of the function, which is a single add instruction.
    // Line 19-21: An overflow check on the addition, which will trap if the addition overflowed.
    // Line 22: Moving the result of the addition into the return register (register 1)
    // Line 23: Returning from the function body, which goes back to line 12
    // Line 12-15: Handling the return data, moving the return value into the right place in memory and stopping execution.
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 24
18: sp[3] = u32 add sp[1], sp[2]
19: sp[4] = u32 lt_eq sp[1], sp[3]
20: jump if sp[4] to 22
21: call 30
22: sp[1] = sp[3]
23: return
24: @4 = const u32 30791
25: @3 = u32 lt @0, @4
26: jump if @3 to 29
27: @1 = indirect const u64 15764276373176857197
28: trap &[@1; @2]
29: return
30: @1 = indirect const u64 14990209321349310352
31: trap &[@1; @2]
32: return

    ");
}

// Tests Brillig u32 subtraction code-gen. It includes underflow check
#[test]
fn brillig_sub() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = sub v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 24
18: sp[3] = u32 sub sp[1], sp[2]
19: sp[4] = u32 lt_eq sp[2], sp[1]
20: jump if sp[4] to 22
21: call 30
22: sp[1] = sp[3]
23: return
24: @4 = const u32 30791
25: @3 = u32 lt @0, @4
26: jump if @3 to 29
27: @1 = indirect const u64 15764276373176857197
28: trap &[@1; @2]
29: return
30: @1 = indirect const u64 1998584279744703196
31: trap &[@1; @2]
32: return

    ");
}

// Tests Brillig u32 multiplication code-gen. It includes overflow check
#[test]
fn brillig_mul() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = mul v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 28
18: sp[3] = u32 mul sp[1], sp[2]
19: sp[5] = const u32 0
20: sp[4] = u32 eq sp[5], sp[2]
21: jump if sp[4] to 26
22: sp[7] = u32 div sp[3], sp[2]
23: sp[6] = u32 eq sp[7], sp[1]
24: jump if sp[6] to 26
25: call 34
26: sp[1] = sp[3]
27: return
28: @4 = const u32 30791
29: @3 = u32 lt @0, @4
30: jump if @3 to 33
31: @1 = indirect const u64 15764276373176857197
32: trap &[@1; @2]
33: return
34: @1 = indirect const u64 361444214588792908
35: trap &[@1; @2]
36: return

    ");
}

// Tests Brillig u32 division code-gen.
#[test]
fn brillig_div() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = div v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 div sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 modulo operation code-gen.
#[test]
fn brillig_mod() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = mod v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 23
18: sp[4] = u32 div sp[1], sp[2]
19: sp[5] = u32 mul sp[4], sp[2]
20: sp[3] = u32 sub sp[1], sp[5]
21: sp[1] = sp[3]
22: return
23: @4 = const u32 30791
24: @3 = u32 lt @0, @4
25: jump if @3 to 28
26: @1 = indirect const u64 15764276373176857197
27: trap &[@1; @2]
28: return

    ");
}

// Tests Brillig u32 equality comparison code-gen.
#[test]
fn brillig_eq() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u1):
        v4 = call f1(v0, v1) -> u1
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 1
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 1
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 eq sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 less than comparison code-gen.
#[test]
fn brillig_lt() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u1):
        v4 = call f1(v0, v1) -> u1
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = lt v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 1
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 1
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 lt sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 bitwise AND code-gen.
#[test]
fn brillig_and() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = and v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 and sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 bitwise OR code-gen.
#[test]
fn brillig_or() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = or v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 or sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 bitwise XOR code-gen.
#[test]
fn brillig_xor() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = xor v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 xor sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 left shift code-gen.
#[test]
fn brillig_shl() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = shl v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 shl sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig u32 right shift code-gen.
#[test]
fn brillig_shr() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = call f1(v0, v1) -> u32
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = shr v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BLACKBOX::RANGE input: w1, bits: 32
BLACKBOX::RANGE input: w2, bits: 32
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
BLACKBOX::RANGE input: w3, bits: 32
ASSERT w3 = w2

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
17: call 21
18: sp[3] = u32 shr sp[1], sp[2]
19: sp[1] = sp[3]
20: return
21: @4 = const u32 30791
22: @3 = u32 lt @0, @4
23: jump if @3 to 26
24: @1 = indirect const u64 15764276373176857197
25: trap &[@1; @2]
26: return

    ");
}

// Tests Brillig Field addition.
#[test]
fn brillig_add_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field, v2: Field):
        v4 = call f1(v0, v1) -> Field
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
ASSERT w3 = w2

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
16: sp[3] = field add sp[1], sp[2]
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

// Tests Brillig Field subtraction.
#[test]
fn brillig_sub_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field, v2: Field):
        v4 = call f1(v0, v1) -> Field
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = sub v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
ASSERT w3 = w2

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
16: sp[3] = field sub sp[1], sp[2]
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

// Tests Brillig Field multiplication
#[test]
fn brillig_mul_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field, v2: Field):
        v4 = call f1(v0, v1) -> Field
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = mul v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
ASSERT w3 = w2

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
16: sp[3] = field mul sp[1], sp[2]
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

// Tests Brillig Field division
#[test]
fn brillig_div_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field, v2: Field):
        v4 = call f1(v0, v1) -> Field
        constrain v4 == v2
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = div v0, v1
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0, w1, w2]
public parameters: []
return values: []
BRILLIG CALL func: 0, inputs: [w0, w1], outputs: [w3]
ASSERT w3 = w2

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
16: sp[3] = field field_div sp[1], sp[2]
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
