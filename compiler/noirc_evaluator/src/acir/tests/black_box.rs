//! A blackbox predicate can be a non-constant SSA value which only folds to a compile-time
//! zero during ACIR gen (e.g. `eq (x + 5), x`). Such calls are statically disabled, and
//! backends reject a constant zero predicate, so ACIR gen resolves them to the outputs the ACVM
//! produces for a disabled call and emits no opcode.

use acvm::{
    acir::circuit::{
        Opcode,
        opcodes::{BlackBoxFuncCall, FunctionInput},
    },
    assert_circuit_snapshot,
};

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn multi_scalar_mul_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field):
        v1 = add v0, Field 5
        v2 = eq v1, v0
        v3 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860] : [(Field, Field); 1]
        v4 = make_array [v0, Field 0] : [(Field, Field); 1]
        v5 = call multi_scalar_mul(v3, v4, v2) -> [(Field, Field); 1]
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1, w2]
    ASSERT w1 = 0
    ASSERT w2 = 0
    ");
}

#[test]
fn multi_scalar_mul_with_witness_predicate_is_emitted() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field, v1: u1):
        v3 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860] : [(Field, Field); 1]
        v4 = make_array [v0, Field 0] : [(Field, Field); 1]
        v5 = call multi_scalar_mul(v3, v4, v1) -> [(Field, Field); 1]
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2, w3]
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w4 = 0
    BLACKBOX::MULTI_SCALAR_MUL points: [1, 17631683881184975370165255887551781615748388533673675138860], scalars: [w0, w4], predicate: w1, outputs: [w5, w6]
    ASSERT w2 = w5
    ASSERT w3 = w6
    ");
}

#[test]
fn embedded_curve_add_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field):
        v1 = add v0, Field 5
        v2 = eq v1, v0
        v3 = call embedded_curve_add(Field 1, Field 17631683881184975370165255887551781615748388533673675138860, v0, v0, v2) -> [(Field, Field); 1]
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1, w2]
    ASSERT w1 = 0
    ASSERT w2 = 0
    ");
}

#[test]
fn embedded_curve_add_with_witness_predicate_is_emitted() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field, v1: u1):
        v3 = call embedded_curve_add(Field 1, Field 17631683881184975370165255887551781615748388533673675138860, v0, v0, v1) -> [(Field, Field); 1]
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2, w3]
    BLACKBOX::RANGE input: w1, bits: 1
    BLACKBOX::EMBEDDED_CURVE_ADD input1: [1, 17631683881184975370165255887551781615748388533673675138860], input2: [w0, w0], predicate: w1, outputs: [w4, w5]
    ASSERT w2 = w4
    ASSERT w3 = w5
    ");
}

#[test]
fn ecdsa_secp256k1_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = ecdsa_src("ecdsa_secp256k1", "v5 = add v0, Field 5\n        v6 = eq v5, v0");
    let program = ssa_to_acir_program(&src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w1 = 1
    ");
}

#[test]
fn ecdsa_secp256r1_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = ecdsa_src("ecdsa_secp256r1", "v5 = add v0, Field 5\n        v6 = eq v5, v0");
    let program = ssa_to_acir_program(&src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w1 = 1
    ");
}

#[test]
fn ecdsa_secp256k1_with_witness_predicate_is_emitted() {
    let src = ecdsa_src("ecdsa_secp256k1", "v6 = eq v0, Field 5");
    let program = ssa_to_acir_program(&src);

    let opcodes = &program.functions[0].opcodes;
    let ecdsa_predicates: Vec<_> = opcodes
        .iter()
        .filter_map(|opcode| match opcode {
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::EcdsaSecp256k1 { predicate, .. }) => {
                Some(*predicate)
            }
            _ => None,
        })
        .collect();
    assert!(
        matches!(ecdsa_predicates.as_slice(), [FunctionInput::Witness(_)]),
        "expected one ECDSA opcode with a witness predicate, got: {ecdsa_predicates:?}"
    );
}

#[test]
fn recursive_aggregation_with_side_effects_folded_to_zero_emits_no_opcode() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: [Field; 1], v2: [Field; 1], v3: [Field; 1]):
        v4 = add v0, Field 5
        v5 = eq v4, v0
        enable_side_effects v5
        call recursive_aggregation(v1, v2, v3, v0, u32 1)
        enable_side_effects u1 1
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: []
    ");
}

#[test]
fn recursive_aggregation_with_witness_side_effects_is_emitted() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: [Field; 1], v2: [Field; 1], v3: [Field; 1], v4: u1):
        enable_side_effects v4
        call recursive_aggregation(v1, v2, v3, v0, u32 1)
        enable_side_effects u1 1
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w4, bits: 1
    BLACKBOX::RECURSIVE_AGGREGATION verification_key: [w1], proof: [w2], public_inputs: [w3], key_hash: w0, proof_type: 1, predicate: w4
    ");
}

/// SSA calling the ECDSA blackbox `name` with constant inputs and the predicate `v6`, which is
/// defined by `predicate_def` in terms of the parameter `v0`.
fn ecdsa_src(name: &str, predicate_def: &str) -> String {
    let bytes = |len: usize| vec!["u8 1"; len].join(", ");
    let key = bytes(32);
    let signature = bytes(64);
    format!(
        "
    acir(inline) predicate_pure fn main f0 {{
      b0(v0: Field):
        {predicate_def}
        v1 = make_array [{key}] : [u8; 32]
        v2 = make_array [{key}] : [u8; 32]
        v3 = make_array [{signature}] : [u8; 64]
        v4 = make_array [{key}] : [u8; 32]
        v7 = call {name}(v1, v2, v3, v4, v6) -> u1
        return v7
    }}
    "
    )
}
