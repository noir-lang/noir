///! This module contains functions for producing a higher level view disassembler of Brillig.
use super::BrilligBinaryOp;
use crate::brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE;
use acvm::acir::brillig_vm::{BinaryFieldOp, BinaryIntOp, RegisterIndex, RegisterOrMemory, Value};

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
<<<<<<< HEAD
        if *self == ReservedRegisters::stack_pointer() {
            "Stack".into()
        } else {
            format!("R{}", self.to_usize())
        }
=======
        format!("R{}", self.to_usize())
>>>>>>> kw/x86-call-instruction
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
<<<<<<< HEAD
                    format!("i{}::{}", bit_size, op.debug_to_string())
=======
                    format!("{}:{}", op.debug_to_string(), bit_size)
>>>>>>> kw/x86-call-instruction
                }
            }
            BrilligBinaryOp::Modulo { is_signed_integer, bit_size } => {
                let op = if *is_signed_integer { "%" } else { "%%" };
<<<<<<< HEAD
                // rationale: if there's >= 64 bits, we should not bother with this detail
=======
>>>>>>> kw/x86-call-instruction
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

<<<<<<< HEAD
impl DebugToString for RegisterValueOrArray {
    fn debug_to_string(&self) -> String {
        match self {
            RegisterValueOrArray::RegisterIndex(index) => index.debug_to_string(),
            RegisterValueOrArray::HeapArray(index, size) => {
                format!("{}[0..{}]", index.debug_to_string(), size)
            }
=======
impl DebugToString for RegisterOrMemory {
    fn debug_to_string(&self) -> String {
        match self {
            RegisterOrMemory::RegisterIndex(index) => index.debug_to_string(),
            RegisterOrMemory::HeapArray(index, size) => {
                format!("{}[0..{}]", index.debug_to_string(), size)
            }
            RegisterOrMemory::HeapVector(index, size_register) => {
                format!("{}[0..*R{}]", index.debug_to_string(), size_register.debug_to_string())
            }
>>>>>>> kw/x86-call-instruction
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
pub(crate) fn constrain_instruction(condition: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  ASSERT {} != 0", condition);
=======
    debug_println!("ASSERT R{} == 0", condition);
>>>>>>> kw/x86-call-instruction
}

/// Processes a return instruction.
pub(crate) fn return_instruction(return_registers: &[RegisterIndex]) {
<<<<<<< HEAD
    let registers_string = return_registers
        .iter()
        .map(RegisterIndex::debug_to_string)
        .collect::<Vec<String>>()
        .join(", ");

    debug_println!("  // return {};", registers_string);
=======
    for (destination_index, return_register) in return_registers.iter().enumerate() {
        debug_println!("MOV R{}, R{}", destination_index, *return_register);
    }
    debug_println!("STOP");
>>>>>>> kw/x86-call-instruction
}

/// Emits a `mov` instruction.
pub(crate) fn mov_instruction(destination: RegisterIndex, source: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  MOV {}, {}", destination, source);
=======
    debug_println!("MOV R{}, R{}", destination, source);
>>>>>>> kw/x86-call-instruction
}

/// Processes a binary instruction according `operation`.
pub(crate) fn binary_instruction(
    lhs: RegisterIndex,
    rhs: RegisterIndex,
    result: RegisterIndex,
    operation: BrilligBinaryOp,
) {
<<<<<<< HEAD
    debug_println!("  {} = {} {} {}", result, lhs, operation, rhs);
=======
    debug_println!("{} = {} {} {}", result, lhs, operation, rhs);
>>>>>>> kw/x86-call-instruction
}

/// Stores the value of `constant` in the `result` register
pub(crate) fn const_instruction(result: RegisterIndex, constant: Value) {
<<<<<<< HEAD
    debug_println!("  CONST {} = {}", result, constant);
=======
    debug_println!("CONST {} = {}", result, constant);
>>>>>>> kw/x86-call-instruction
}

/// Processes a not instruction. Append with "_" as this is a high-level instruction.
pub(crate) fn not_instruction(condition: RegisterIndex, result: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  _NOT {} = !{}", result, condition);
=======
    debug_println!("_NOT {} = !R{}", result, condition);
>>>>>>> kw/x86-call-instruction
}

/// Processes a foreign call instruction.
pub(crate) fn foreign_call_instruction(
    func_name: String,
<<<<<<< HEAD
    inputs: &[RegisterValueOrArray],
    outputs: &[RegisterValueOrArray],
) {
    debug_println!("  FOREIGN_CALL {} ({}) => {}", func_name, inputs, outputs);
=======
    inputs: &[RegisterOrMemory],
    outputs: &[RegisterOrMemory],
) {
    debug_println!("FOREIGN_CALL {} ({}) => {}", func_name, inputs, outputs);
>>>>>>> kw/x86-call-instruction
}

/// Emits a load instruction
pub(crate) fn load_instruction(destination: RegisterIndex, source_pointer: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  LOAD {} = *{}", destination, source_pointer);
=======
    debug_println!("LOAD R{} = *R{}", destination, source_pointer);
>>>>>>> kw/x86-call-instruction
}

/// Emits a store instruction
pub(crate) fn store_instruction(destination_pointer: RegisterIndex, source: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  STORE *{} = {}", destination_pointer, source);
=======
    debug_println!("STORE *{} = {}", destination_pointer, source);
>>>>>>> kw/x86-call-instruction
}

/// Emits a stop instruction
pub(crate) fn stop_instruction() {
<<<<<<< HEAD
    debug_println!("  STOP");
=======
    debug_println!("STOP");
}

/// Adds a unresolved external `Call` instruction to the bytecode.
pub(crate) fn add_external_call_instruction(func_label: String) {
    debug_println!("CALL {}", func_label);
}

/// Debug function for allocate_fixed_length_array instruction
pub(crate) fn allocate_fixed_length_array(pointer_register: RegisterIndex, size: usize) {
    debug_println!("ALLOCATE_FIXED_LENGTH_ARRAY {} = {}", pointer_register, size);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for allocate_array_instruction
pub(crate) fn allocate_array_instruction(
    pointer_register: RegisterIndex,
    size_register: RegisterIndex,
) {
<<<<<<< HEAD
    debug_println!("  ALLOCATE_ARRAY {} SIZE {}", pointer_register, size_register);
=======
    debug_println!("ALLOCATE_ARRAY {} SIZE {}", pointer_register, size_register);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for array_get
pub(crate) fn array_get(array_ptr: RegisterIndex, index: RegisterIndex, result: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  ARRAY_GET {}[{}] -> {}", array_ptr, index, result);
=======
    debug_println!("ARRAY_GET {}[{}] -> {}", array_ptr, index, result);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for array_set
pub(crate) fn array_set(array_ptr: RegisterIndex, index: RegisterIndex, value: RegisterIndex) {
<<<<<<< HEAD
    debug_println!("  ARRAY_SET {}[{}] = {}", array_ptr, index, value);
=======
    debug_println!("ARRAY_SET {}[{}] = {}", array_ptr, index, value);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for copy_array_instruction
pub(crate) fn copy_array_instruction(
    source: RegisterIndex,
    destination: RegisterIndex,
    num_elements_register: RegisterIndex,
) {
<<<<<<< HEAD
    debug_println!(
        "  COPY_ARRAY {} -> {} ({} ELEMENTS)",
        source,
        destination,
        num_elements_register
    );
=======
    debug_println!("COPY_ARRAY {} -> {} ({} ELEMENTS)", source, destination, num_elements_register);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for enter_context
pub(crate) fn enter_context(label: String) {
<<<<<<< HEAD
    if !label.ends_with("-b0") {
        // Hacky readability fix: don't print labels e.g. f1 then f1-b0 one after another, they mean the same thing
        debug_println!("{}:", label);
    }
=======
    debug_println!("ENTER_CONTEXT {}", label);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for jump_instruction
pub(crate) fn jump_instruction(target_label: String) {
<<<<<<< HEAD
    debug_println!("  JUMP_TO {}", target_label);
=======
    debug_println!("JUMP_TO {}", target_label);
>>>>>>> kw/x86-call-instruction
}

/// Debug function for jump_if_instruction
pub(crate) fn jump_if_instruction<T: ToString>(condition: RegisterIndex, target_label: T) {
<<<<<<< HEAD
    debug_println!("  JUMP_IF {} TO {}", condition, target_label.to_string());
=======
    debug_println!("JUMP_IF {} TO {}", condition, target_label.to_string());
>>>>>>> kw/x86-call-instruction
}

/// Debug function for cast_instruction
pub(crate) fn cast_instruction(
    destination: RegisterIndex,
    source: RegisterIndex,
    target_bit_size: u32,
) {
<<<<<<< HEAD
    debug_println!("  CAST {} FROM {} TO {} BITS", destination, source, target_bit_size);
=======
    debug_println!("CAST {} FROM {} TO {} BITS", destination, source, target_bit_size);
>>>>>>> kw/x86-call-instruction
}
