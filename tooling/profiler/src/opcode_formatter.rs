use acir::circuit::{directives::Directive, opcodes::BlackBoxFuncCall, Opcode};

fn format_blackbox_function<F>(call: &BlackBoxFuncCall<F>) -> String {
    match call {
        BlackBoxFuncCall::AES128Encrypt { .. } => "aes128_encrypt".to_string(),
        BlackBoxFuncCall::AND { .. } => "and".to_string(),
        BlackBoxFuncCall::XOR { .. } => "xor".to_string(),
        BlackBoxFuncCall::RANGE { .. } => "range".to_string(),
        BlackBoxFuncCall::SHA256 { .. } => "sha256".to_string(),
        BlackBoxFuncCall::Blake2s { .. } => "blake2s".to_string(),
        BlackBoxFuncCall::Blake3 { .. } => "blake3".to_string(),
        BlackBoxFuncCall::SchnorrVerify { .. } => "schnorr_verify".to_string(),
        BlackBoxFuncCall::PedersenCommitment { .. } => "pedersen_commitment".to_string(),
        BlackBoxFuncCall::PedersenHash { .. } => "pedersen_hash".to_string(),
        BlackBoxFuncCall::EcdsaSecp256k1 { .. } => "ecdsa_secp256k1".to_string(),
        BlackBoxFuncCall::EcdsaSecp256r1 { .. } => "ecdsa_secp256r1".to_string(),
        BlackBoxFuncCall::MultiScalarMul { .. } => "multi_scalar_mul".to_string(),
        BlackBoxFuncCall::EmbeddedCurveAdd { .. } => "embedded_curve_add".to_string(),
        BlackBoxFuncCall::Keccak256 { .. } => "keccak256".to_string(),
        BlackBoxFuncCall::Keccakf1600 { .. } => "keccakf1600".to_string(),
        BlackBoxFuncCall::RecursiveAggregation { .. } => "recursive_aggregation".to_string(),
        BlackBoxFuncCall::BigIntAdd { .. } => "big_int_add".to_string(),
        BlackBoxFuncCall::BigIntSub { .. } => "big_int_sub".to_string(),
        BlackBoxFuncCall::BigIntMul { .. } => "big_int_mul".to_string(),
        BlackBoxFuncCall::BigIntDiv { .. } => "big_int_div".to_string(),
        BlackBoxFuncCall::BigIntFromLeBytes { .. } => "big_int_from_le_bytes".to_string(),
        BlackBoxFuncCall::BigIntToLeBytes { .. } => "big_int_to_le_bytes".to_string(),
        BlackBoxFuncCall::Poseidon2Permutation { .. } => "poseidon2_permutation".to_string(),
        BlackBoxFuncCall::Sha256Compression { .. } => "sha256_compression".to_string(),
    }
}

fn format_directive_kind<F>(directive: &Directive<F>) -> String {
    match directive {
        Directive::ToLeRadix { .. } => "to_le_radix".to_string(),
    }
}

fn format_opcode_kind<F>(opcode: &Opcode<F>) -> String {
    match opcode {
        Opcode::AssertZero(_) => "arithmetic".to_string(),
        Opcode::BlackBoxFuncCall(call) => format!("blackbox::{}", format_blackbox_function(call)),
        Opcode::MemoryOp { .. } => "memory::op".to_string(),
        Opcode::MemoryInit { .. } => "memory::init".to_string(),
        Opcode::Directive(directive) => format!("directive::{}", format_directive_kind(directive)),
        Opcode::BrilligCall { .. } => "brillig_call".to_string(),
        Opcode::Call { .. } => "acir_call".to_string(),
    }
}

pub(crate) fn format_opcode<F>(opcode: &Opcode<F>) -> String {
    format!("opcode::{}", format_opcode_kind(opcode))
}
