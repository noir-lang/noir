//! This module contains functions for producing a higher level view disassembler of Brillig.

use super::{BrilligBinaryOp, artifact::Label};
use crate::brillig::brillig_ir::ReservedRegisters;
use acvm::{
    FieldElement,
    acir::brillig::{BlackBoxOp, HeapArray, HeapVector, MemoryAddress, ValueOrArray},
};

/// Trait for converting values into debug-friendly strings.
pub(crate) trait DebugToString {
    fn debug_to_string(&self) -> String;
}

macro_rules! default_to_string_impl {
    ($($t:ty)*) => ($(
        impl DebugToString for $t {
            fn debug_to_string(&self) -> String {
                self.to_string()
            }
        }
    )*)
}

default_to_string_impl! { str usize u32 }

impl DebugToString for MemoryAddress {
    fn debug_to_string(&self) -> String {
        if *self == ReservedRegisters::free_memory_pointer() {
            "FreeMem".into()
        } else if *self == ReservedRegisters::stack_pointer() {
            "StackPointer".into()
        } else {
            match self {
                MemoryAddress::Direct(address) => format!("M{address}"),
                MemoryAddress::Relative(offset) => format!("S{offset}"),
            }
        }
    }
}

impl DebugToString for HeapArray {
    fn debug_to_string(&self) -> String {
        format!("{}[0..{}]", self.pointer.debug_to_string(), self.size)
    }
}

impl DebugToString for HeapVector {
    fn debug_to_string(&self) -> String {
        format!("{}[0..{}]", self.pointer.debug_to_string(), self.size.debug_to_string())
    }
}

impl DebugToString for BrilligBinaryOp {
    fn debug_to_string(&self) -> String {
        match self {
            BrilligBinaryOp::Add => "+".into(),
            BrilligBinaryOp::Sub => "-".into(),
            BrilligBinaryOp::Mul => "*".into(),
            BrilligBinaryOp::Equals => "==".into(),
            BrilligBinaryOp::FieldDiv => "f/".into(),
            BrilligBinaryOp::UnsignedDiv => "/".into(),
            BrilligBinaryOp::LessThan => "<".into(),
            BrilligBinaryOp::LessThanEquals => "<=".into(),
            BrilligBinaryOp::And => "&".into(),
            BrilligBinaryOp::Or => "|".into(),
            BrilligBinaryOp::Xor => "^".into(),
            BrilligBinaryOp::Shl => "<<".into(),
            BrilligBinaryOp::Shr => ">>".into(),
        }
    }
}

impl DebugToString for Label {
    fn debug_to_string(&self) -> String {
        self.to_string()
    }
}

impl DebugToString for FieldElement {
    fn debug_to_string(&self) -> String {
        self.to_string()
    }
}

impl DebugToString for ValueOrArray {
    fn debug_to_string(&self) -> String {
        match self {
            ValueOrArray::MemoryAddress(index) => index.debug_to_string(),
            ValueOrArray::HeapArray(heap_array) => heap_array.debug_to_string(),
            ValueOrArray::HeapVector(vector) => vector.debug_to_string(),
        }
    }
}

impl DebugToString for BlackBoxOp {
    fn debug_to_string(&self) -> String {
        match self {
            BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => format!(
                "AES128 ENCRYPT {} {} {}  -> {}",
                inputs.debug_to_string(),
                iv.debug_to_string(),
                key.debug_to_string(),
                outputs.debug_to_string()
            ),
            BlackBoxOp::Keccakf1600 { input, output } => {
                format!("KECCAKF1600 {} -> {}", input.debug_to_string(), output.debug_to_string())
            }
            BlackBoxOp::Blake2s { message, output } => {
                format!("BLAKE2S {} -> {}", message.debug_to_string(), output.debug_to_string())
            }
            BlackBoxOp::Blake3 { message, output } => {
                format!("BLAKE3 {} -> {}", message.debug_to_string(), output.debug_to_string())
            }
            BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => format!(
                "ECDSA_SECP256K1 {} {} {} {} -> {}",
                hashed_msg.debug_to_string(),
                public_key_x.debug_to_string(),
                public_key_y.debug_to_string(),
                signature.debug_to_string(),
                result.debug_to_string()
            ),
            BlackBoxOp::EcdsaSecp256r1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => format!(
                "ECDSA_SECP256R1 {} {} {} {} -> {}",
                hashed_msg.debug_to_string(),
                public_key_x.debug_to_string(),
                public_key_y.debug_to_string(),
                signature.debug_to_string(),
                result.debug_to_string()
            ),
            BlackBoxOp::MultiScalarMul { points, scalars, outputs } => format!(
                "MULTI_SCALAR_MUL {} {} -> {}",
                points.debug_to_string(),
                scalars.debug_to_string(),
                outputs.debug_to_string()
            ),
            BlackBoxOp::EmbeddedCurveAdd { input1_x, input1_y, input2_x, input2_y, result } => {
                format!(
                    "EMBEDDED_CURVE_ADD ({} {}) ({} {}) -> {}",
                    input1_x.debug_to_string(),
                    input1_y.debug_to_string(),
                    input2_x.debug_to_string(),
                    input2_y.debug_to_string(),
                    result.debug_to_string()
                )
            }
            BlackBoxOp::Poseidon2Permutation { message, output } => format!(
                "POSEIDON2_PERMUTATION {} -> {}",
                message.debug_to_string(),
                output.debug_to_string()
            ),
            BlackBoxOp::Sha256Compression { input, hash_values, output } => format!(
                "SHA256COMPRESSION {} {} -> {}",
                input.debug_to_string(),
                hash_values.debug_to_string(),
                output.debug_to_string()
            ),
            BlackBoxOp::ToRadix { input, radix, output_pointer, num_limbs, output_bits: _ } => {
                format!(
                    "TO_RADIX {} {} {} -> {}",
                    input.debug_to_string(),
                    radix.debug_to_string(),
                    num_limbs.debug_to_string(),
                    output_pointer.debug_to_string()
                )
            }
        }
    }
}

impl<T: DebugToString> DebugToString for [T] {
    fn debug_to_string(&self) -> String {
        self.iter().map(|x| x.debug_to_string()).collect::<Vec<String>>().join(", ")
    }
}

macro_rules! debug_println {
    ( $enable_debug:expr, $literal:expr ) => {
        if $enable_debug {
            println!("{}", $literal)
        }
    };
    ( $enable_debug:expr, $format_message:expr, $( $x:expr ),* ) => {
        if $enable_debug {
            println!($format_message, $( $x.debug_to_string(), )*)
        }
    };
}

pub(crate) struct DebugShow {
    enable_debug_trace: bool,
}

impl DebugShow {
    pub(crate) fn new(enable_debug_trace: bool) -> DebugShow {
        DebugShow { enable_debug_trace }
    }

    /// Emits a `trap` instruction.
    pub(crate) fn trap_instruction(&self, revert_data: HeapVector) {
        debug_println!(self.enable_debug_trace, "  TRAP {}", revert_data);
    }

    /// Emits a `mov` instruction.
    pub(crate) fn mov_instruction(&self, destination: MemoryAddress, source: MemoryAddress) {
        debug_println!(self.enable_debug_trace, "  MOV {}, {}", destination, source);
    }

    /// Emits a `conditional mov` instruction.
    pub(crate) fn conditional_mov_instruction(
        &self,
        destination: MemoryAddress,
        source_then: MemoryAddress,
        source_else: MemoryAddress,
        condition: MemoryAddress,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  {} = MOV if {} then {}, else {}",
            destination,
            condition,
            source_then,
            source_else
        );
    }

    /// Emits a `cast` instruction.
    pub(crate) fn cast_instruction(
        &self,
        destination: MemoryAddress,
        source: MemoryAddress,
        bit_size: u32,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  CAST {}, {} as u{}",
            destination,
            source,
            bit_size
        );
    }

    /// Processes a binary instruction according `operation`.
    pub(crate) fn binary_instruction(
        &self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
        operation: BrilligBinaryOp,
    ) {
        debug_println!(self.enable_debug_trace, "  {} = {} {} {}", result, lhs, operation, rhs);
    }

    /// Processes an unsigned modulo instruction.
    pub(crate) fn modulo_instruction(
        &self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) {
        debug_println!(self.enable_debug_trace, "  {} = {} % {}", result, lhs, rhs);
    }

    /// Stores the value of `constant` in the `result` register.
    pub(crate) fn const_instruction<F: DebugToString>(&self, result: MemoryAddress, constant: F) {
        debug_println!(self.enable_debug_trace, "  CONST {} = {}", result, constant);
    }

    /// Stores the value of `constant` in the register pointed at by `result_pointer`.
    pub(crate) fn indirect_const_instruction<F: DebugToString>(
        &self,
        result_pointer: MemoryAddress,
        constant: F,
    ) {
        debug_println!(self.enable_debug_trace, "  ICONST {} = {}", result_pointer, constant);
    }

    /// Processes a not instruction. Append with "_" as this is a high-level instruction.
    pub(crate) fn not_instruction(
        &self,
        condition: MemoryAddress,
        bit_size: u32,
        result: MemoryAddress,
    ) {
        debug_println!(self.enable_debug_trace, "  i{}_NOT {} = !{}", bit_size, result, condition);
    }

    /// Processes a foreign call instruction.
    pub(crate) fn foreign_call_instruction(
        &self,
        func_name: &str,
        inputs: &[ValueOrArray],
        outputs: &[ValueOrArray],
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  FOREIGN_CALL {} ({}) => {}",
            func_name,
            inputs,
            outputs
        );
    }

    /// Emits a load instruction
    pub(crate) fn load_instruction(
        &self,
        destination: MemoryAddress,
        source_pointer: MemoryAddress,
    ) {
        debug_println!(self.enable_debug_trace, "  LOAD {} = *{}", destination, source_pointer);
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &self,
        destination_pointer: MemoryAddress,
        source: MemoryAddress,
    ) {
        debug_println!(self.enable_debug_trace, "  STORE *{} = {}", destination_pointer, source);
    }

    /// Emits a return instruction
    pub(crate) fn return_instruction(&self) {
        debug_println!(self.enable_debug_trace, "  RETURN");
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&self, return_data: HeapVector) {
        debug_println!(self.enable_debug_trace, "  STOP {}", return_data);
    }

    /// Debug function for `enter_context`
    pub(crate) fn enter_context(&self, label: &Label) {
        if !self.enable_debug_trace {
            return;
        }
        let label = label.to_string();
        // Hacky readability fix: don't print labels e.g. f1 then f1-b0 one after another, they mean the same thing
        if !label.ends_with("-b0") {
            println!("{label}:");
        }
    }

    /// Debug function for `jump_instruction`
    pub(crate) fn jump_instruction(&self, target_label: &Label) {
        debug_println!(self.enable_debug_trace, "  JUMP_TO {}", target_label);
    }

    /// Debug function for `jump_if_instruction`
    pub(crate) fn jump_if_instruction(&self, condition: MemoryAddress, target_label: &Label) {
        debug_println!(self.enable_debug_trace, "  JUMP_IF {} TO {}", condition, target_label);
    }

    /// Debug function for `black_box_op`
    pub(crate) fn black_box_op_instruction(&self, op: &BlackBoxOp) {
        debug_println!(self.enable_debug_trace, "  {}", op);
    }

    /// Debug function for `cast_instruction`
    pub(crate) fn add_external_call_instruction(&self, func_label: &Label) {
        debug_println!(self.enable_debug_trace, "  CALL {}", func_label);
    }

    /// Debug function for `calldata_copy`
    pub(crate) fn calldata_copy_instruction(
        &self,
        destination: MemoryAddress,
        calldata_size: usize,
        offset: usize,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  CALLDATA_COPY {} {}..{}",
            destination,
            offset,
            offset + calldata_size
        );
    }
}

#[cfg(test)]
mod tests {
    use acvm::acir::brillig::{BlackBoxOp, HeapArray, MemoryAddress, lengths::SemiFlattenedLength};

    use super::DebugToString;
    use crate::brillig::brillig_ir::ReservedRegisters;

    fn array(offset: u32, size: u32) -> HeapArray {
        HeapArray { pointer: MemoryAddress::relative(offset), size: SemiFlattenedLength(size) }
    }

    /// The `--show-brillig` trace format for black box ops is user-visible output.
    #[test]
    fn black_box_op_trace_format() {
        let r = MemoryAddress::relative;
        let cases = [
            (
                BlackBoxOp::AES128Encrypt {
                    inputs: array(1, 16),
                    iv: array(2, 16),
                    key: array(3, 16),
                    outputs: array(4, 16),
                },
                "AES128 ENCRYPT S1[0..16] S2[0..16] S3[0..16]  -> S4[0..16]",
            ),
            (
                BlackBoxOp::Keccakf1600 { input: array(1, 25), output: array(2, 25) },
                "KECCAKF1600 S1[0..25] -> S2[0..25]",
            ),
            (
                BlackBoxOp::Blake2s { message: array(1, 8), output: array(2, 32) },
                "BLAKE2S S1[0..8] -> S2[0..32]",
            ),
            (
                BlackBoxOp::Blake3 { message: array(1, 8), output: array(2, 32) },
                "BLAKE3 S1[0..8] -> S2[0..32]",
            ),
            (
                BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg: array(1, 32),
                    public_key_x: array(2, 32),
                    public_key_y: array(3, 32),
                    signature: array(4, 64),
                    result: r(5),
                },
                "ECDSA_SECP256K1 S1[0..32] S2[0..32] S3[0..32] S4[0..64] -> S5",
            ),
            (
                BlackBoxOp::EcdsaSecp256r1 {
                    hashed_msg: array(1, 32),
                    public_key_x: array(2, 32),
                    public_key_y: array(3, 32),
                    signature: array(4, 64),
                    result: r(5),
                },
                "ECDSA_SECP256R1 S1[0..32] S2[0..32] S3[0..32] S4[0..64] -> S5",
            ),
            (
                BlackBoxOp::MultiScalarMul {
                    points: array(1, 6),
                    scalars: array(2, 4),
                    outputs: array(3, 3),
                },
                "MULTI_SCALAR_MUL S1[0..6] S2[0..4] -> S3[0..3]",
            ),
            (
                BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: r(1),
                    input1_y: r(2),
                    input2_x: r(3),
                    input2_y: r(4),
                    result: array(5, 3),
                },
                "EMBEDDED_CURVE_ADD (S1 S2) (S3 S4) -> S5[0..3]",
            ),
            (
                BlackBoxOp::Poseidon2Permutation { message: array(1, 4), output: array(2, 4) },
                "POSEIDON2_PERMUTATION S1[0..4] -> S2[0..4]",
            ),
            (
                BlackBoxOp::Sha256Compression {
                    input: array(1, 16),
                    hash_values: array(2, 8),
                    output: array(3, 8),
                },
                "SHA256COMPRESSION S1[0..16] S2[0..8] -> S3[0..8]",
            ),
            (
                BlackBoxOp::ToRadix {
                    input: r(1),
                    radix: r(2),
                    output_pointer: r(3),
                    num_limbs: r(4),
                    output_bits: r(5),
                },
                "TO_RADIX S1 S2 S4 -> S3",
            ),
            (
                BlackBoxOp::Blake2s {
                    message: HeapArray {
                        pointer: ReservedRegisters::free_memory_pointer(),
                        size: SemiFlattenedLength(8),
                    },
                    output: HeapArray {
                        pointer: MemoryAddress::direct(10),
                        size: SemiFlattenedLength(32),
                    },
                },
                "BLAKE2S FreeMem[0..8] -> M10[0..32]",
            ),
        ];
        for (op, expected) in cases {
            assert_eq!(op.debug_to_string(), expected);
        }
    }
}
