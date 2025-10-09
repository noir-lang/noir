use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

// Tests array element access by index code-gen for Brillig.
#[test]
fn brillig_array_get() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = call f1(v0) -> u32
        constrain v2 == u32 10
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_get v1, index v0 -> u32
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BRILLIG CALL func: 0, inputs: [w0], outputs: [w1]
ASSERT w1 = 10

unconstrained func 0: foo
 0: @2 = const u32 1
 1: @1 = const u32 32838
 2: @0 = const u32 70
 3: sp[2] = const u32 1
 4: sp[3] = const u32 0
 5: @68 = calldata copy [sp[3]; sp[2]]
 6: @68 = cast @68 to u32
 7: sp[1] = @68
 8: call 14
 9: call 15
10: @69 = sp[1]
11: sp[2] = const u32 69
12: sp[3] = const u32 1
13: stop &[sp[2]; sp[3]]
14: return
15: call 35
16: sp[2] = const u32 10
17: sp[3] = const u32 20
18: sp[4] = const u32 30
19: sp[5] = @1
20: sp[6] = const u32 4
21: @1 = u32 add @1, sp[6]
22: sp[5] = indirect const u32 1
23: sp[6] = u32 add sp[5], @2
24: sp[7] = sp[6]
25: store sp[2] at sp[7]
26: sp[7] = u32 add sp[7], @2
27: store sp[3] at sp[7]
28: sp[7] = u32 add sp[7], @2
29: store sp[4] at sp[7]
30: sp[3] = u32 add sp[5], @2
31: sp[4] = u32 add sp[3], sp[1]
32: sp[2] = load sp[4]
33: sp[1] = sp[2]
34: return
35: @4 = const u32 30790
36: @3 = u32 lt @0, @4
37: jump if @3 to 40
38: @1 = indirect const u64 15764276373176857197
39: trap &[@1; @2]
40: return

    ");
}

// Tests setting an array element and retrieving it
#[test]
fn brillig_array_set() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = call f1(v0) -> u32
        constrain v2 == u32 99
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_set v1, index v0, value u32 99
        v3 = array_get v2, index v0 -> u32
        return v3
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
func 0
private parameters: [w0]
public parameters: []
return values: []
BLACKBOX::RANGE input: w0, bits: 32
BRILLIG CALL func: 0, inputs: [w0], outputs: [w1]
ASSERT w1 = 99

unconstrained func 0: foo
 0: @2 = const u32 1
 1: @1 = const u32 32838
 2: @0 = const u32 70
 3: sp[2] = const u32 1
 4: sp[3] = const u32 0
 5: @68 = calldata copy [sp[3]; sp[2]]
 6: @68 = cast @68 to u32
 7: sp[1] = @68
 8: call 14
 9: call 15
10: @69 = sp[1]
11: sp[2] = const u32 69
12: sp[3] = const u32 1
13: stop &[sp[2]; sp[3]]
14: return
15: call 43
16: sp[2] = const u32 10
17: sp[3] = const u32 20
18: sp[4] = const u32 30
19: sp[5] = @1
20: sp[6] = const u32 4
21: @1 = u32 add @1, sp[6]
22: sp[5] = indirect const u32 1
23: sp[6] = u32 add sp[5], @2
24: sp[7] = sp[6]
25: store sp[2] at sp[7]
26: sp[7] = u32 add sp[7], @2
27: store sp[3] at sp[7]
28: sp[7] = u32 add sp[7], @2
29: store sp[4] at sp[7]
30: sp[2] = const u32 99
31: @3 = sp[5]
32: @4 = const u32 4
33: call 49
34: sp[3] = @5
35: sp[4] = u32 add sp[3], @2
36: sp[6] = u32 add sp[4], sp[1]
37: store sp[2] at sp[6]
38: sp[4] = u32 add sp[3], @2
39: sp[5] = u32 add sp[4], sp[1]
40: sp[2] = load sp[5]
41: sp[1] = sp[2]
42: return
43: @4 = const u32 30790
44: @3 = u32 lt @0, @4
45: jump if @3 to 48
46: @1 = indirect const u64 15764276373176857197
47: trap &[@1; @2]
48: return
49: @6 = load @3
50: @7 = u32 eq @6, @2
51: jump if @7 to 53
52: jump to 55
53: @5 = @3
54: jump to 69
55: @5 = @1
56: @1 = u32 add @1, @4
57: @9 = u32 add @3, @4
58: @10 = @3
59: @11 = @5
60: @12 = u32 eq @10, @9
61: jump if @12 to 67
62: @8 = load @10
63: store @8 at @11
64: @10 = u32 add @10, @2
65: @11 = u32 add @11, @2
66: jump to 60
67: @5 = indirect const u32 1
68: @6 = u32 sub @6, @2
69: return

    ");
}

// Tests array operations with reference counting inc_rc/dec_rc
#[test]
fn brillig_array_with_rc_ops() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = call f1() -> u32
        constrain v1 == u32 99
        return
    }

    brillig(inline) fn foo f1 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        inc_rc v0
        v1 = array_set v0, index u32 0, value u32 99
        dec_rc v0
        v2 = array_get v1, index u32 0 -> u32
        return v2
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
ASSERT w0 = 99

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
13: call 46
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
29: sp[1] = u32 add sp[1], @2
30: store sp[1] at sp[4]
31: sp[1] = const u32 0
32: sp[2] = const u32 99
33: @3 = sp[4]
34: @4 = const u32 4
35: call 52
36: sp[3] = @5
37: sp[5] = u32 add sp[3], sp[1]
38: store sp[2] at sp[5]
39: sp[2] = load sp[4]
40: sp[2] = u32 sub sp[2], @2
41: store sp[2] at sp[4]
42: sp[4] = u32 add sp[3], sp[1]
43: sp[2] = load sp[4]
44: sp[1] = sp[2]
45: return
46: @4 = const u32 30789
47: @3 = u32 lt @0, @4
48: jump if @3 to 51
49: @1 = indirect const u64 15764276373176857197
50: trap &[@1; @2]
51: return
52: @6 = load @3
53: @7 = u32 eq @6, @2
54: jump if @7 to 56
55: jump to 58
56: @5 = @3
57: jump to 72
58: @5 = @1
59: @1 = u32 add @1, @4
60: @9 = u32 add @3, @4
61: @10 = @3
62: @11 = @5
63: @12 = u32 eq @10, @9
64: jump if @12 to 70
65: @8 = load @10
66: store @8 at @11
67: @10 = u32 add @10, @2
68: @11 = u32 add @11, @2
69: jump to 63
70: @5 = indirect const u32 1
71: @6 = u32 sub @6, @2
72: return

    ");
}
