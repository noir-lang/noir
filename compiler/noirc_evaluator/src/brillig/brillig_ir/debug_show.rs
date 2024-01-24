//! This module contains functions for producing a higher level view disassembler of Brillig.

use super::BrilligBinaryOp;
use crate::brillig::brillig_ir::{ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE};
use acvm::acir::brillig::{
    BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, HeapVector, RegisterIndex, RegisterOrMemory,
    Value,
};

/// Trait for converting values into debug-friendly strings.
trait DebugToString {
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

impl DebugToString for RegisterIndex {
    fn debug_to_string(&self) -> String {
        if *self == ReservedRegisters::stack_pointer() {
            "Stack".into()
        } else if *self == ReservedRegisters::previous_stack_pointer() {
            "PrevStack".into()
        } else {
            format!("R{}", self.to_usize())
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

impl DebugToString for BinaryFieldOp {
    fn debug_to_string(&self) -> String {
        match self {
            BinaryFieldOp::Add => "f+".into(),
            BinaryFieldOp::Sub => "f-".into(),
            BinaryFieldOp::Mul => "f*".into(),
            BinaryFieldOp::Div => "f/".into(),
            BinaryFieldOp::Equals => "f==".into(),
        }
    }
}

impl DebugToString for BinaryIntOp {
    fn debug_to_string(&self) -> String {
        match self {
            BinaryIntOp::Add => "+".into(),
            BinaryIntOp::Sub => "-".into(),
            BinaryIntOp::Mul => "*".into(),
            BinaryIntOp::Equals => "==".into(),
            BinaryIntOp::SignedDiv => "/".into(),
            BinaryIntOp::UnsignedDiv => "//".into(),
            BinaryIntOp::LessThan => "<".into(),
            BinaryIntOp::LessThanEquals => "<=".into(),
            BinaryIntOp::And => "&&".into(),
            BinaryIntOp::Or => "||".into(),
            BinaryIntOp::Xor => "^".into(),
            BinaryIntOp::Shl | BinaryIntOp::Shr => {
                unreachable!("bit shift should have been replaced")
            }
        }
    }
}

impl DebugToString for BrilligBinaryOp {
    fn debug_to_string(&self) -> String {
        match self {
            BrilligBinaryOp::Field { op } => op.debug_to_string(),
            BrilligBinaryOp::Integer { op, bit_size } => {
                // rationale: if there's >= 64 bits, we should not bother with this detail
                if *bit_size >= BRILLIG_MEMORY_ADDRESSING_BIT_SIZE {
                    op.debug_to_string()
                } else {
                    format!("i{}::{}", bit_size, op.debug_to_string())
                }
            }
            BrilligBinaryOp::Modulo { is_signed_integer, bit_size } => {
                let op = if *is_signed_integer { "%" } else { "%%" };
                // rationale: if there's >= 64 bits, we should not bother with this detail
                if *bit_size >= BRILLIG_MEMORY_ADDRESSING_BIT_SIZE {
                    op.into()
                } else {
                    format!("{op}:{bit_size}")
                }
            }
        }
    }
}

impl DebugToString for Value {
    fn debug_to_string(&self) -> String {
        self.to_usize().to_string()
    }
}

impl DebugToString for RegisterOrMemory {
    fn debug_to_string(&self) -> String {
        match self {
            RegisterOrMemory::RegisterIndex(index) => index.debug_to_string(),
            RegisterOrMemory::HeapArray(heap_array) => heap_array.debug_to_string(),
            RegisterOrMemory::HeapVector(vector) => vector.debug_to_string(),
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

    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(&self, condition: RegisterIndex) {
        debug_println!(self.enable_debug_trace, "  ASSERT {} != 0", condition);
    }

    /// Processes a return instruction.
    pub(crate) fn return_instruction(&self, return_registers: &[RegisterIndex]) {
        let registers_string = return_registers
            .iter()
            .map(RegisterIndex::debug_to_string)
            .collect::<Vec<String>>()
            .join(", ");

        debug_println!(self.enable_debug_trace, "  // return {};", registers_string);
    }

    /// Emits a `mov` instruction.
    pub(crate) fn mov_instruction(&self, destination: RegisterIndex, source: RegisterIndex) {
        debug_println!(self.enable_debug_trace, "  MOV {}, {}", destination, source);
    }

    /// Processes a binary instruction according `operation`.
    pub(crate) fn binary_instruction(
        &self,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
        operation: BrilligBinaryOp,
    ) {
        debug_println!(self.enable_debug_trace, "  {} = {} {} {}", result, lhs, operation, rhs);
    }

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction(&self, result: RegisterIndex, constant: Value) {
        debug_println!(self.enable_debug_trace, "  CONST {} = {}", result, constant);
    }

    /// Processes a not instruction. Append with "_" as this is a high-level instruction.
    pub(crate) fn not_instruction(
        &self,
        condition: RegisterIndex,
        bit_size: u32,
        result: RegisterIndex,
    ) {
        debug_println!(self.enable_debug_trace, "  i{}_NOT {} = !{}", bit_size, result, condition);
    }

    /// Processes a foreign call instruction.
    pub(crate) fn foreign_call_instruction(
        &self,
        func_name: String,
        inputs: &[RegisterOrMemory],
        outputs: &[RegisterOrMemory],
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
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    ) {
        debug_println!(self.enable_debug_trace, "  LOAD {} = *{}", destination, source_pointer);
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &self,
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    ) {
        debug_println!(self.enable_debug_trace, "  STORE *{} = {}", destination_pointer, source);
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&self) {
        debug_println!(self.enable_debug_trace, "  STOP");
    }

    /// Debug function for allocate_array_instruction
    pub(crate) fn allocate_array_instruction(
        &self,
        pointer_register: RegisterIndex,
        size_register: RegisterIndex,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  ALLOCATE_ARRAY {} SIZE {}",
            pointer_register,
            size_register
        );
    }

    /// Debug function for allocate_instruction
    pub(crate) fn allocate_instruction(&self, pointer_register: RegisterIndex) {
        debug_println!(self.enable_debug_trace, "  ALLOCATE {} ", pointer_register);
    }

    /// Debug function for array_get
    pub(crate) fn array_get(
        &self,
        array_ptr: RegisterIndex,
        index: RegisterIndex,
        result: RegisterIndex,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  ARRAY_GET {}[{}] -> {}",
            array_ptr,
            index,
            result
        );
    }

    /// Debug function for array_set
    pub(crate) fn array_set(
        &self,
        array_ptr: RegisterIndex,
        index: RegisterIndex,
        value: RegisterIndex,
    ) {
        debug_println!(self.enable_debug_trace, "  ARRAY_SET {}[{}] = {}", array_ptr, index, value);
    }

    /// Debug function for copy_array_instruction
    pub(crate) fn copy_array_instruction(
        &self,
        source: RegisterIndex,
        destination: RegisterIndex,
        num_elements_register: RegisterIndex,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  COPY_ARRAY {} -> {} ({} ELEMENTS)",
            source,
            destination,
            num_elements_register
        );
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
        condition: RegisterIndex,
        target_label: T,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  JUMP_IF {} TO {}",
            condition,
            target_label.to_string()
        );
    }

    /// Debug function for cast_instruction
    pub(crate) fn truncate_instruction(
        &self,
        destination: RegisterIndex,
        source: RegisterIndex,
        target_bit_size: u32,
    ) {
        debug_println!(
            self.enable_debug_trace,
            "  TRUNCATE {} FROM {} TO {} BITS",
            destination,
            source,
            target_bit_size
        );
    }

    /// Debug function for black_box_op
    pub(crate) fn black_box_op_instruction(&self, op: BlackBoxOp) {
        match op {
            BlackBoxOp::Sha256 { message, output } => {
                debug_println!(self.enable_debug_trace, "  SHA256 {} -> {}", message, output);
            }
            BlackBoxOp::Keccak256 { message, output } => {
                debug_println!(self.enable_debug_trace, "  KECCAK256 {} -> {}", message, output);
            }
            BlackBoxOp::Keccakf1600 { message, output } => {
                debug_println!(self.enable_debug_trace, "  KECCAKF1600 {} -> {}", message, output);
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
            BlackBoxOp::FixedBaseScalarMul { low, high, result } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  FIXED_BASE_SCALAR_MUL {} {} -> {}",
                    low,
                    high,
                    result
                );
            }
            BlackBoxOp::EmbeddedCurveAdd { input1_x, input1_y, input2_x, input2_y, result } => {
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
            BlackBoxOp::EmbeddedCurveDouble { input1_x, input1_y, result } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  EMBEDDED_CURVE_DOUBLE ({} {}) -> {}",
                    input1_x,
                    input1_y,
                    result
                );
            }
            BlackBoxOp::PedersenCommitment { inputs, domain_separator, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  PEDERSEN {} {} -> {}",
                    inputs,
                    domain_separator,
                    output
                );
            }
            BlackBoxOp::PedersenHash { inputs, domain_separator, output } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  PEDERSEN_HASH {} {} -> {}",
                    inputs,
                    domain_separator,
                    output
                );
            }
            BlackBoxOp::SchnorrVerify {
                public_key_x,
                public_key_y,
                message,
                signature,
                result,
            } => {
                debug_println!(
                    self.enable_debug_trace,
                    "  SCHNORR_VERIFY {} {} {} {} -> {}",
                    public_key_x,
                    public_key_y,
                    message,
                    signature,
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
            BlackBoxOp::BigIntNeg { lhs, rhs, output } => {
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
        }
    }

    /// Debug function for cast_instruction
    pub(crate) fn add_external_call_instruction(&self, func_label: String) {
        debug_println!(self.enable_debug_trace, "  CALL {}", func_label);
    }
}
