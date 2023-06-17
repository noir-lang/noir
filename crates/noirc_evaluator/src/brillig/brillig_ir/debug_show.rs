///! This module contains functions for producing a higher level view disassembler of Brillig.
use super::BrilligBinaryOp;
use crate::brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE;
use acvm::acir::brillig_vm::{
    BinaryFieldOp, BinaryIntOp, RegisterIndex, RegisterValueOrArray, Value,
};

/// Controls whether debug traces are enabled
const ENABLE_DEBUG_TRACE: bool = false;

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
        format!("R{}", self.to_usize().to_string())
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
            BinaryIntOp::Shl => "<<".into(),
            BinaryIntOp::Shr => ">>".into(),
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
                    format!("{}:{}", op.debug_to_string(), bit_size)
                }
            }
            BrilligBinaryOp::Modulo { is_signed_integer, bit_size } => {
                let op = if *is_signed_integer { "%" } else { "%%" };
                if *bit_size >= BRILLIG_MEMORY_ADDRESSING_BIT_SIZE {
                    op.into()
                } else {
                    format!("{}:{}", op, bit_size)
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

impl DebugToString for RegisterValueOrArray {
    fn debug_to_string(&self) -> String {
        match self {
            RegisterValueOrArray::RegisterIndex(index) => index.debug_to_string(),
            RegisterValueOrArray::HeapArray(index, size) => {
                format!("{}[0..{}]", index.debug_to_string(), size)
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
    ( $first:expr ) => {
        if ENABLE_DEBUG_TRACE {
            println!("{}", $first);
        }
    };
    ( $first:expr, $( $x:expr ),* ) => {
        if ENABLE_DEBUG_TRACE {
            println!($first, $( $x.debug_to_string(), )*)
        }
    };
}

/// Emits brillig bytecode to jump to a trap condition if `condition`
/// is false.
pub fn constrain_instruction(condition: RegisterIndex) {
    debug_println!("ASSERT R{} == 0", condition);
}

/// Processes a return instruction.
pub fn return_instruction(return_registers: &[RegisterIndex]) {
    for (destination_index, return_register) in return_registers.iter().enumerate() {
        debug_println!("MOV R{}, R{}", destination_index, *return_register);
    }
    debug_println!("STOP");
}

/// Emits a `mov` instruction.
pub fn mov_instruction(destination: RegisterIndex, source: RegisterIndex) {
    debug_println!("MOV R{}, R{}", destination, source);
}

/// Processes a binary instruction according `operation`.
pub fn binary_instruction(
    lhs: RegisterIndex,
    rhs: RegisterIndex,
    result: RegisterIndex,
    operation: BrilligBinaryOp,
) {
    debug_println!("{} = {} {} {}", result, lhs, operation, rhs);
}

/// Stores the value of `constant` in the `result` register
pub fn const_instruction(result: RegisterIndex, constant: Value) {
    debug_println!("CONST {} = {}", result, constant);
}

/// Processes a not instruction. Append with "_" as this is a high-level instruction.
pub fn not_instruction(condition: RegisterIndex, result: RegisterIndex) {
    debug_println!("_NOT {} = !R{}", result, condition);
}

/// Processes a foreign call instruction.
pub fn foreign_call_instruction(
    func_name: String,
    inputs: &[RegisterValueOrArray],
    outputs: &[RegisterValueOrArray],
) {
    debug_println!("FOREIGN_CALL {} ({}) => {}", func_name, inputs, outputs);
}

/// Emits a load instruction
pub fn load_instruction(destination: RegisterIndex, source_pointer: RegisterIndex) {
    debug_println!("LOAD R{} = *R{}", destination, source_pointer);
}

/// Emits a store instruction
pub fn store_instruction(destination_pointer: RegisterIndex, source: RegisterIndex) {
    debug_println!("STORE *{} = {}", destination_pointer, source);
}

/// Emits a stop instruction
pub fn stop_instruction() {
    debug_println!("STOP");
}

/// Adds a unresolved external `Call` instruction to the bytecode.
pub fn add_external_call_instruction<T: ToString>(func_label: T) {
    debug_println!("CALL {}", func_label.to_string());
}

/// Debug function for allocate_fixed_length_array instruction
pub fn allocate_fixed_length_array(pointer_register: RegisterIndex, size: usize) {
    debug_println!("ALLOCATE_FIXED_LENGTH_ARRAY {} = {}", pointer_register, size);
}

/// Debug function for allocate_array_instruction
pub fn allocate_array_instruction(pointer_register: RegisterIndex, size_register: RegisterIndex) {
    debug_println!("ALLOCATE_ARRAY {} SIZE {}", pointer_register, size_register);
}

/// Debug function for array_get
pub fn array_get(array_ptr: RegisterIndex, index: RegisterIndex, result: RegisterIndex) {
    debug_println!("ARRAY_GET {}[{}] -> {}", array_ptr, index, result);
}

/// Debug function for array_set
pub fn array_set(array_ptr: RegisterIndex, index: RegisterIndex, value: RegisterIndex) {
    debug_println!("ARRAY_SET {}[{}] = {}", array_ptr, index, value);
}

/// Debug function for copy_array_instruction
pub fn copy_array_instruction(
    source: RegisterIndex,
    destination: RegisterIndex,
    num_elements_register: RegisterIndex,
) {
    debug_println!("COPY_ARRAY {} -> {} ({} ELEMENTS)", source, destination, num_elements_register);
}

/// Debug function for enter_context
pub fn enter_context<T: ToString>(label: T) {
    debug_println!("ENTER_CONTEXT {}", label.to_string());
}

/// Debug function for jump_instruction
pub fn jump_instruction<T: ToString>(target_label: T) {
    debug_println!("JUMP_TO {}", target_label.to_string());
}

/// Debug function for jump_if_instruction
pub fn jump_if_instruction<T: ToString>(condition: RegisterIndex, target_label: T) {
    debug_println!("JUMP_IF {} TO {}", condition, target_label.to_string());
}

/// Debug function for cast_instruction
pub fn cast_instruction(destination: RegisterIndex, source: RegisterIndex, target_bit_size: u32) {
    debug_println!("CAST {} FROM {} TO {} BITS", destination, source, target_bit_size);
}
