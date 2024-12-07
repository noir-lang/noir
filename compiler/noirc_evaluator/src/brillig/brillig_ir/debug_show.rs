//! This module contains functions for producing a higher level view disassembler of Brillig.

use super::BrilligBinaryOp;
use crate::brillig::brillig_ir::ReservedRegisters;
use acvm::{
    acir::brillig::{BlackBoxOp, HeapArray, HeapVector, MemoryAddress, ValueOrArray},
    FieldElement,
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
                MemoryAddress::Direct(address) => format!("M{}", address),
                MemoryAddress::Relative(offset) => format!("S{}", offset),
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
            BrilligBinaryOp::Modulo => "%".into(),
        }
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

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction<F: DebugToString>(&self, result: MemoryAddress, constant: F) {
        debug_println!(self.enable_debug_trace, "  CONST {} = {}", result, constant);
    }

    /// Stores the value of `constant` in the `result` register
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
        func_name: String,
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

    /// Debug function for enter_context
    pub(crate) fn enter_context(&self, label: String) {
        if !label.ends_with("-b0") {
            // Hacky readability fix: don't print labels e.g. f1 then f1-b0 one after another, they mean the same thing
            debug_println!(self.enable_debug_trace, "{}:", label);
        }
    }

    /// Debug function for jump_instruction
    pub(crate) fn jump_instruction(&self, target_label: String) {
        debug_println!(self.enable_debug_trace, "  JUMP_TO {}", target_label);
    }

    /// Debug function for jump_if_instruction
    pub(crate) fn jump_if_instruction<T: ToString>(
        &self,
        condition: MemoryAddress,
        target_label: T,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  JUMP_IF {} TO {}",
            condition,
            target_label.to_string()
        );
    }

    /// Debug function for black_box_op
    pub(crate) fn black_box_op_instruction(&self, op: &BlackBoxOp) {
        match op {
            BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  AES128 ENCRYPT {} {} {}  -> {}",
                    inputs,
                    iv,
                    key,
                    outputs
                );
            }
            BlackBoxOp::Keccakf1600 { input, output } => {
                debug_println!(self.enable_debug_trace, "  KECCAKF1600 {} -> {}", input, output);
            }
            BlackBoxOp::Blake2s { message, output } => {
                debug_println!(self.enable_debug_trace, "  BLAKE2S {} -> {}", message, output);
            }
            BlackBoxOp::Blake3 { message, output } => {
                debug_println!(self.enable_debug_trace, "  BLAKE3 {} -> {}", message, output);
            }
            BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  ECDSA_SECP256K1 {} {} {} {} -> {}",
                    hashed_msg,
                    public_key_x,
                    public_key_y,
                    signature,
                    result
                );
            }
            BlackBoxOp::EcdsaSecp256r1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  ECDSA_SECP256R1 {} {} {} {} -> {}",
                    hashed_msg,
                    public_key_x,
                    public_key_y,
                    signature,
                    result
                );
            }
            BlackBoxOp::MultiScalarMul { points, scalars, outputs } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  MULTI_SCALAR_MUL {} {} -> {}",
                    points,
                    scalars,
                    outputs
                );
            }
            BlackBoxOp::EmbeddedCurveAdd {
                input1_x, input1_y, input2_x, input2_y, result, ..
            } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  EMBEDDED_CURVE_ADD ({} {}) ({} {}) -> {}",
                    input1_x,
                    input1_y,
                    input2_x,
                    input2_y,
                    result
                );
            }
            BlackBoxOp::BigIntAdd { lhs, rhs, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_ADD {} {} -> {}",
                    lhs,
                    rhs,
                    output
                );
            }
            BlackBoxOp::BigIntSub { lhs, rhs, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_NEG {} {} -> {}",
                    lhs,
                    rhs,
                    output
                );
            }
            BlackBoxOp::BigIntMul { lhs, rhs, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_MUL {} {} -> {}",
                    lhs,
                    rhs,
                    output
                );
            }
            BlackBoxOp::BigIntDiv { lhs, rhs, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_DIV {} {} -> {}",
                    lhs,
                    rhs,
                    output
                );
            }
            BlackBoxOp::BigIntFromLeBytes { inputs, modulus, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_FROM_LE_BYTES {} {} -> {}",
                    inputs,
                    modulus,
                    output
                );
            }
            BlackBoxOp::BigIntToLeBytes { input, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  BIGINT_TO_LE_BYTES {} -> {}",
                    input,
                    output
                );
            }
            BlackBoxOp::Poseidon2Permutation { message, output, len } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  POSEIDON2_PERMUTATION {} {} -> {}",
                    message,
                    len,
                    output
                );
            }
            BlackBoxOp::Sha256Compression { input, hash_values, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  SHA256COMPRESSION {} {} -> {}",
                    input,
                    hash_values,
                    output
                );
            }
            BlackBoxOp::ToRadix { input, radix, output, output_bits: _ } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  TO_RADIX {} {} -> {}",
                    input,
                    radix,
                    output
                );
            }
        }
    }

    /// Debug function for cast_instruction
    pub(crate) fn add_external_call_instruction(&self, func_label: String) {
        debug_println!(self.enable_debug_trace, "  CALL {}", func_label);
    }

    /// Debug function for calldata_copy
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
