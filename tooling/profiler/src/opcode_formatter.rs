use acir::brillig::{BinaryFieldOp, BinaryIntOp, BlackBoxOp, Opcode as BrilligOpcode};
use acir::circuit::{directives::Directive, opcodes::BlackBoxFuncCall, Opcode as AcirOpcode};
use acir::AcirField;

#[derive(Debug)]
pub(crate) enum AcirOrBrilligOpcode<F: AcirField> {
    Acir(AcirOpcode<F>),
    Brillig(BrilligOpcode<F>),
}

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

fn format_blackbox_op(call: &BlackBoxOp) -> String {
    match call {
        BlackBoxOp::AES128Encrypt { .. } => "aes128_encrypt".to_string(),
        BlackBoxOp::Sha256 { .. } => "sha256".to_string(),
        BlackBoxOp::Blake2s { .. } => "blake2s".to_string(),
        BlackBoxOp::Blake3 { .. } => "blake3".to_string(),
        BlackBoxOp::SchnorrVerify { .. } => "schnorr_verify".to_string(),
        BlackBoxOp::PedersenCommitment { .. } => "pedersen_commitment".to_string(),
        BlackBoxOp::PedersenHash { .. } => "pedersen_hash".to_string(),
        BlackBoxOp::EcdsaSecp256k1 { .. } => "ecdsa_secp256k1".to_string(),
        BlackBoxOp::EcdsaSecp256r1 { .. } => "ecdsa_secp256r1".to_string(),
        BlackBoxOp::MultiScalarMul { .. } => "multi_scalar_mul".to_string(),
        BlackBoxOp::EmbeddedCurveAdd { .. } => "embedded_curve_add".to_string(),
        BlackBoxOp::Keccak256 { .. } => "keccak256".to_string(),
        BlackBoxOp::Keccakf1600 { .. } => "keccakf1600".to_string(),
        BlackBoxOp::BigIntAdd { .. } => "big_int_add".to_string(),
        BlackBoxOp::BigIntSub { .. } => "big_int_sub".to_string(),
        BlackBoxOp::BigIntMul { .. } => "big_int_mul".to_string(),
        BlackBoxOp::BigIntDiv { .. } => "big_int_div".to_string(),
        BlackBoxOp::BigIntFromLeBytes { .. } => "big_int_from_le_bytes".to_string(),
        BlackBoxOp::BigIntToLeBytes { .. } => "big_int_to_le_bytes".to_string(),
        BlackBoxOp::Poseidon2Permutation { .. } => "poseidon2_permutation".to_string(),
        BlackBoxOp::Sha256Compression { .. } => "sha256_compression".to_string(),
        BlackBoxOp::ToRadix { .. } => "to_radix".to_string(),
    }
}

fn format_directive_kind<F>(directive: &Directive<F>) -> String {
    match directive {
        Directive::ToLeRadix { .. } => "to_le_radix".to_string(),
    }
}

fn format_acir_opcode_kind<F>(opcode: &AcirOpcode<F>) -> String {
    match opcode {
        AcirOpcode::AssertZero(_) => "arithmetic".to_string(),
        AcirOpcode::BlackBoxFuncCall(call) => {
            format!("blackbox::{}", format_blackbox_function(call))
        }
        AcirOpcode::MemoryOp { .. } => "memory::op".to_string(),
        AcirOpcode::MemoryInit { .. } => "memory::init".to_string(),
        AcirOpcode::Directive(directive) => {
            format!("directive::{}", format_directive_kind(directive))
        }
        AcirOpcode::BrilligCall { id, .. } => format!("brillig_call({id})"),
        AcirOpcode::Call { .. } => "acir_call".to_string(),
    }
}

fn format_binary_field_op(op: &BinaryFieldOp) -> String {
    match op {
        BinaryFieldOp::Add => "add".to_string(),
        BinaryFieldOp::Sub => "sub".to_string(),
        BinaryFieldOp::Mul => "mul".to_string(),
        BinaryFieldOp::Div => "fdiv".to_string(),
        BinaryFieldOp::IntegerDiv => "div".to_string(),
        BinaryFieldOp::Equals => "eq".to_string(),
        BinaryFieldOp::LessThan => "lt".to_string(),
        BinaryFieldOp::LessThanEquals => "lte".to_string(),
    }
}

fn format_binary_int(op: &acir::brillig::BinaryIntOp) -> String {
    match op {
        BinaryIntOp::Add => "add".to_string(),
        BinaryIntOp::Sub => "sub".to_string(),
        BinaryIntOp::Mul => "mul".to_string(),
        BinaryIntOp::Div => "div".to_string(),
        BinaryIntOp::Equals => "eq".to_string(),
        BinaryIntOp::LessThan => "lt".to_string(),
        BinaryIntOp::LessThanEquals => "lte".to_string(),
        BinaryIntOp::And => "and".to_string(),
        BinaryIntOp::Or => "or".to_string(),
        BinaryIntOp::Xor => "xor".to_string(),
        BinaryIntOp::Shl => "shl".to_string(),
        BinaryIntOp::Shr => "shr".to_string(),
    }
}

fn format_brillig_opcode_kind<F>(opcode: &BrilligOpcode<F>) -> String {
    match opcode {
        BrilligOpcode::BinaryFieldOp { op, .. } => format!("field::{}", format_binary_field_op(op)),
        BrilligOpcode::BinaryIntOp { op, bit_size, .. } => {
            format!("{bit_size}::{}", format_binary_int(op))
        }
        BrilligOpcode::BlackBox(func) => format!("blackbox::{}", format_blackbox_op(func)),
        BrilligOpcode::Call { .. } => "call".to_string(),
        BrilligOpcode::CalldataCopy { .. } => "calldata_copy".to_string(),
        BrilligOpcode::Cast { .. } => "cast".to_string(),
        BrilligOpcode::ConditionalMov { .. } => "cmov".to_string(),
        BrilligOpcode::Const { .. } => "const".to_string(),
        BrilligOpcode::ForeignCall { function, .. } => format!("foreign_call({})", function),
        BrilligOpcode::Jump { .. } => "jump".to_string(),
        BrilligOpcode::JumpIf { .. } => "jump_if".to_string(),
        BrilligOpcode::JumpIfNot { .. } => "jump_if_not".to_string(),
        BrilligOpcode::Load { .. } => "load".to_string(),
        BrilligOpcode::Mov { .. } => "mov".to_string(),
        BrilligOpcode::Return => "return".to_string(),
        BrilligOpcode::Stop { .. } => "stop".to_string(),
        BrilligOpcode::Store { .. } => "store".to_string(),
        BrilligOpcode::Trap { .. } => "trap".to_string(),
    }
}

pub(crate) fn format_opcode<F: AcirField>(opcode: &AcirOrBrilligOpcode<F>) -> String {
    match opcode {
        AcirOrBrilligOpcode::Acir(opcode) => format!("acir::{}", format_acir_opcode_kind(opcode)),
        AcirOrBrilligOpcode::Brillig(opcode) => {
            format!("brillig::{}", format_brillig_opcode_kind(opcode))
        }
    }
}
