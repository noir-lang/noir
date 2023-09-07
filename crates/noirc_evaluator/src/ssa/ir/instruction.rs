use acvm::{acir::BlackBoxFunc, FieldElement};
use iter_extended::vecmap;
use num_bigint::BigUint;

use super::{
    basic_block::BasicBlockId,
    dfg::{CallStack, DataFlowGraph},
    map::Id,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

mod call;

use call::simplify_call;

/// Reference to an instruction
///
/// Note that InstructionIds are not unique. That is, two InstructionIds
/// may refer to the same Instruction data. This is because, although
/// identical, instructions may have different results based on their
/// placement within a block.
pub(crate) type InstructionId = Id<Instruction>;

/// These are similar to built-ins in other languages.
/// These can be classified under two categories:
/// - Opcodes which the IR knows the target machine has
/// special support for. (LowLevel)
/// - Opcodes which have no function definition in the
/// source code and must be processed by the IR. An example
/// of this is println.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Intrinsic {
    Sort,
    ArrayLen,
    AssertConstant,
    SlicePushBack,
    SlicePushFront,
    SlicePopBack,
    SlicePopFront,
    SliceInsert,
    SliceRemove,
    StrAsBytes,
    ToBits(Endian),
    ToRadix(Endian),
    BlackBox(BlackBoxFunc),
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::Sort => write!(f, "arraysort"),
            Intrinsic::ArrayLen => write!(f, "array_len"),
            Intrinsic::AssertConstant => write!(f, "assert_constant"),
            Intrinsic::SlicePushBack => write!(f, "slice_push_back"),
            Intrinsic::SlicePushFront => write!(f, "slice_push_front"),
            Intrinsic::SlicePopBack => write!(f, "slice_pop_back"),
            Intrinsic::SlicePopFront => write!(f, "slice_pop_front"),
            Intrinsic::SliceInsert => write!(f, "slice_insert"),
            Intrinsic::SliceRemove => write!(f, "slice_remove"),
            Intrinsic::StrAsBytes => write!(f, "str_as_bytes"),
            Intrinsic::ToBits(Endian::Big) => write!(f, "to_be_bits"),
            Intrinsic::ToBits(Endian::Little) => write!(f, "to_le_bits"),
            Intrinsic::ToRadix(Endian::Big) => write!(f, "to_be_radix"),
            Intrinsic::ToRadix(Endian::Little) => write!(f, "to_le_radix"),
            Intrinsic::BlackBox(function) => write!(f, "{function}"),
        }
    }
}

impl Intrinsic {
    /// Returns whether the `Intrinsic` has side effects.
    ///
    /// If there are no side effects then the `Intrinsic` can be removed if the result is unused.
    pub(crate) fn has_side_effects(&self) -> bool {
        match self {
            Intrinsic::AssertConstant => true,

            Intrinsic::Sort
            | Intrinsic::ArrayLen
            | Intrinsic::SlicePushBack
            | Intrinsic::SlicePushFront
            | Intrinsic::SlicePopBack
            | Intrinsic::SlicePopFront
            | Intrinsic::SliceInsert
            | Intrinsic::SliceRemove
            | Intrinsic::StrAsBytes
            | Intrinsic::ToBits(_)
            | Intrinsic::ToRadix(_) => false,

            // Some black box functions have side-effects
            Intrinsic::BlackBox(func) => matches!(func, BlackBoxFunc::RecursiveAggregation),
        }
    }

    /// Lookup an Intrinsic by name and return it if found.
    /// If there is no such intrinsic by that name, None is returned.
    pub(crate) fn lookup(name: &str) -> Option<Intrinsic> {
        match name {
            "arraysort" => Some(Intrinsic::Sort),
            "array_len" => Some(Intrinsic::ArrayLen),
            "assert_constant" => Some(Intrinsic::AssertConstant),
            "slice_push_back" => Some(Intrinsic::SlicePushBack),
            "slice_push_front" => Some(Intrinsic::SlicePushFront),
            "slice_pop_back" => Some(Intrinsic::SlicePopBack),
            "slice_pop_front" => Some(Intrinsic::SlicePopFront),
            "slice_insert" => Some(Intrinsic::SliceInsert),
            "slice_remove" => Some(Intrinsic::SliceRemove),
            "str_as_bytes" => Some(Intrinsic::StrAsBytes),
            "to_le_radix" => Some(Intrinsic::ToRadix(Endian::Little)),
            "to_be_radix" => Some(Intrinsic::ToRadix(Endian::Big)),
            "to_le_bits" => Some(Intrinsic::ToBits(Endian::Little)),
            "to_be_bits" => Some(Intrinsic::ToBits(Endian::Big)),
            other => BlackBoxFunc::lookup(other).map(Intrinsic::BlackBox),
        }
    }
}

/// The endian-ness of bits when encoding values as bits in e.g. ToBits or ToRadix
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Endian {
    Big,
    Little,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Instructions are used to perform tasks.
/// The instructions that the IR is able to specify are listed below.
pub(crate) enum Instruction {
    /// Binary Operations like +, -, *, /, ==, !=
    Binary(Binary),

    /// Converts `Value` into Typ
    Cast(ValueId, Type),

    /// Computes a bit wise not
    Not(ValueId),

    /// Truncates `value` to `bit_size`
    Truncate { value: ValueId, bit_size: u32, max_bit_size: u32 },

    /// Constrains two values to be equal to one another.
    Constrain(ValueId, ValueId, Option<String>),

    /// Performs a function call with a list of its arguments.
    Call { func: ValueId, arguments: Vec<ValueId> },

    /// Allocates a region of memory. Note that this is not concerned with
    /// the type of memory, the type of element is determined when loading this memory.
    /// This is used for representing mutable variables and references.
    Allocate,

    /// Loads a value from memory.
    Load { address: ValueId },

    /// Writes a value to memory.
    Store { address: ValueId, value: ValueId },

    /// Provides a context for all instructions that follow up until the next
    /// `EnableSideEffects` is encountered, for stating a condition that determines whether
    /// such instructions are allowed to have side-effects.
    ///
    /// This instruction is only emitted after the cfg flattening pass, and is used to annotate
    /// instruction regions with an condition that corresponds to their position in the CFG's
    /// if-branching structure.
    EnableSideEffects { condition: ValueId },

    /// Retrieve a value from an array at the given index
    ArrayGet { array: ValueId, index: ValueId },

    /// Creates a new array with the new value at the given index. All other elements are identical
    /// to those in the given array. This will not modify the original array.
    ///
    /// An optional length can be provided to enabling handling of dynamic slice indices
    ArraySet { array: ValueId, index: ValueId, value: ValueId, length: Option<ValueId> },
}

impl Instruction {
    /// Returns a binary instruction with the given operator, lhs, and rhs
    pub(crate) fn binary(operator: BinaryOp, lhs: ValueId, rhs: ValueId) -> Instruction {
        Instruction::Binary(Binary { lhs, operator, rhs })
    }

    /// Returns the type that this instruction will return.
    pub(crate) fn result_type(&self) -> InstructionResultType {
        match self {
            Instruction::Binary(binary) => binary.result_type(),
            Instruction::Cast(_, typ) => InstructionResultType::Known(typ.clone()),
            Instruction::Allocate { .. } => InstructionResultType::Known(Type::Reference),
            Instruction::Not(value) | Instruction::Truncate { value, .. } => {
                InstructionResultType::Operand(*value)
            }
            Instruction::ArraySet { array, .. } => InstructionResultType::Operand(*array),
            Instruction::Constrain(..)
            | Instruction::Store { .. }
            | Instruction::EnableSideEffects { .. } => InstructionResultType::None,
            Instruction::Load { .. } | Instruction::ArrayGet { .. } | Instruction::Call { .. } => {
                InstructionResultType::Unknown
            }
        }
    }

    /// True if this instruction requires specifying the control type variables when
    /// inserting this instruction into a DataFlowGraph.
    pub(crate) fn requires_ctrl_typevars(&self) -> bool {
        matches!(self.result_type(), InstructionResultType::Unknown)
    }

    /// Pure `Instructions` are instructions which have no side-effects and results are a function of the inputs only,
    /// i.e. there are no interactions with memory.
    ///
    /// Pure instructions can be replaced with the results of another pure instruction with the same inputs.
    pub(crate) fn is_pure(&self, dfg: &DataFlowGraph) -> bool {
        use Instruction::*;

        match self {
            Binary(_) | Cast(_, _) | Not(_) | ArrayGet { .. } | ArraySet { .. } => true,

            // Unclear why this instruction causes problems.
            Truncate { .. } => false,

            // These either have side-effects or interact with memory
            Constrain(..) | EnableSideEffects { .. } | Allocate | Load { .. } | Store { .. } => {
                false
            }

            Call { func, .. } => match dfg[*func] {
                Value::Intrinsic(intrinsic) => !intrinsic.has_side_effects(),
                _ => false,
            },
        }
    }

    pub(crate) fn has_side_effects(&self, dfg: &DataFlowGraph) -> bool {
        use Instruction::*;

        match self {
            Binary(_)
            | Cast(_, _)
            | Not(_)
            | Truncate { .. }
            | Allocate
            | Load { .. }
            | ArrayGet { .. }
            | ArraySet { .. } => false,

            Constrain(..) | Store { .. } | EnableSideEffects { .. } => true,

            // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
            Call { func, .. } => match dfg[*func] {
                Value::Intrinsic(intrinsic) => intrinsic.has_side_effects(),

                // All foreign functions are treated as having side effects.
                // This is because they can be used to pass information
                // from the ACVM to the external world during execution.
                Value::ForeignFunction(_) => true,

                // We must assume that functions contain a side effect as we cannot inspect more deeply.
                Value::Function(_) => true,

                _ => false,
            },
        }
    }

    /// Maps each ValueId inside this instruction to a new ValueId, returning the new instruction.
    /// Note that the returned instruction is fresh and will not have an assigned InstructionId
    /// until it is manually inserted in a DataFlowGraph later.
    pub(crate) fn map_values(&self, mut f: impl FnMut(ValueId) -> ValueId) -> Instruction {
        match self {
            Instruction::Binary(binary) => Instruction::Binary(Binary {
                lhs: f(binary.lhs),
                rhs: f(binary.rhs),
                operator: binary.operator,
            }),
            Instruction::Cast(value, typ) => Instruction::Cast(f(*value), typ.clone()),
            Instruction::Not(value) => Instruction::Not(f(*value)),
            Instruction::Truncate { value, bit_size, max_bit_size } => Instruction::Truncate {
                value: f(*value),
                bit_size: *bit_size,
                max_bit_size: *max_bit_size,
            },
            Instruction::Constrain(lhs, rhs, assert_message) => {
                Instruction::Constrain(f(*lhs), f(*rhs), assert_message.clone())
            }
            Instruction::Call { func, arguments } => Instruction::Call {
                func: f(*func),
                arguments: vecmap(arguments.iter().copied(), f),
            },
            Instruction::Allocate => Instruction::Allocate,
            Instruction::Load { address } => Instruction::Load { address: f(*address) },
            Instruction::Store { address, value } => {
                Instruction::Store { address: f(*address), value: f(*value) }
            }
            Instruction::EnableSideEffects { condition } => {
                Instruction::EnableSideEffects { condition: f(*condition) }
            }
            Instruction::ArrayGet { array, index } => {
                Instruction::ArrayGet { array: f(*array), index: f(*index) }
            }
            Instruction::ArraySet { array, index, value, length } => Instruction::ArraySet {
                array: f(*array),
                index: f(*index),
                value: f(*value),
                length: length.map(f),
            },
        }
    }

    /// Applies a function to each input value this instruction holds.
    pub(crate) fn for_each_value<T>(&self, mut f: impl FnMut(ValueId) -> T) {
        match self {
            Instruction::Binary(binary) => {
                f(binary.lhs);
                f(binary.rhs);
            }
            Instruction::Call { func, arguments } => {
                f(*func);
                for argument in arguments {
                    f(*argument);
                }
            }
            Instruction::Cast(value, _)
            | Instruction::Not(value)
            | Instruction::Truncate { value, .. }
            | Instruction::Load { address: value } => {
                f(*value);
            }
            Instruction::Constrain(lhs, rhs, _) => {
                f(*lhs);
                f(*rhs);
            }

            Instruction::Store { address, value } => {
                f(*address);
                f(*value);
            }
            Instruction::Allocate { .. } => (),
            Instruction::ArrayGet { array, index } => {
                f(*array);
                f(*index);
            }
            Instruction::ArraySet { array, index, value, length } => {
                f(*array);
                f(*index);
                f(*value);
                length.map(&mut f);
            }
            Instruction::EnableSideEffects { condition } => {
                f(*condition);
            }
        }
    }

    /// Try to simplify this instruction. If the instruction can be simplified to a known value,
    /// that value is returned. Otherwise None is returned.
    ///
    /// The `block` parameter indicates the block this new instruction will be inserted into
    /// after this call.
    pub(crate) fn simplify(&self, dfg: &mut DataFlowGraph, block: BasicBlockId) -> SimplifyResult {
        use SimplifyResult::*;
        match self {
            Instruction::Binary(binary) => binary.simplify(dfg),
            Instruction::Cast(value, typ) => simplify_cast(*value, typ, dfg),
            Instruction::Not(value) => {
                match &dfg[dfg.resolve(*value)] {
                    // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                    // there is no Not on FieldElement, so we'd need to convert between u128. This
                    // would be incorrect however since the extra bits on the field would not be flipped.
                    Value::NumericConstant { constant, typ } if *typ == Type::bool() => {
                        let value = constant.is_zero() as u128;
                        SimplifiedTo(dfg.make_constant(value.into(), Type::bool()))
                    }
                    Value::Instruction { instruction, .. } => {
                        // !!v => v
                        if let Instruction::Not(value) = &dfg[*instruction] {
                            SimplifiedTo(*value)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Instruction::Constrain(lhs, rhs, ..) => {
                if dfg.resolve(*lhs) == dfg.resolve(*rhs) {
                    // Remove trivial case `assert_eq(x, x)`
                    SimplifyResult::Remove
                } else {
                    SimplifyResult::None
                }
            }
            Instruction::ArrayGet { array, index } => {
                let array = dfg.get_array_constant(*array);
                let index = dfg.get_numeric_constant(*index);
                if let (Some((array, _)), Some(index)) = (array, index) {
                    let index =
                        index.try_to_u64().expect("Expected array index to fit in u64") as usize;
                    if index < array.len() {
                        return SimplifiedTo(array[index]);
                    }
                }
                None
            }
            Instruction::ArraySet { array, index, value, .. } => {
                let array = dfg.get_array_constant(*array);
                let index = dfg.get_numeric_constant(*index);
                if let (Some((array, element_type)), Some(index)) = (array, index) {
                    let index =
                        index.try_to_u64().expect("Expected array index to fit in u64") as usize;

                    if index < array.len() {
                        let new_array = dfg.make_array(array.update(index, *value), element_type);
                        return SimplifiedTo(new_array);
                    }
                }
                None
            }
            Instruction::Truncate { value, bit_size, .. } => {
                if let Some((numeric_constant, typ)) = dfg.get_numeric_constant_with_type(*value) {
                    let integer_modulus = 2_u128.pow(*bit_size);
                    let truncated = numeric_constant.to_u128() % integer_modulus;
                    SimplifiedTo(dfg.make_constant(truncated.into(), typ))
                } else {
                    None
                }
            }
            Instruction::Call { func, arguments } => simplify_call(*func, arguments, dfg, block),
            Instruction::EnableSideEffects { condition } => {
                if let Some(last) = dfg[block].instructions().last().copied() {
                    let last = &mut dfg[last];
                    if matches!(last, Instruction::EnableSideEffects { .. }) {
                        *last = Instruction::EnableSideEffects { condition: *condition };
                        return Remove;
                    }
                }
                None
            }
            Instruction::Allocate { .. } => None,
            Instruction::Load { .. } => None,
            Instruction::Store { .. } => None,
        }
    }
}

/// Try to simplify this cast instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
fn simplify_cast(value: ValueId, dst_typ: &Type, dfg: &mut DataFlowGraph) -> SimplifyResult {
    use SimplifyResult::*;
    if let Some(constant) = dfg.get_numeric_constant(value) {
        let src_typ = dfg.type_of_value(value);
        match (src_typ, dst_typ) {
            (Type::Numeric(NumericType::NativeField), Type::Numeric(NumericType::NativeField)) => {
                // Field -> Field: use src value
                SimplifiedTo(value)
            }
            (
                Type::Numeric(NumericType::Unsigned { .. }),
                Type::Numeric(NumericType::NativeField),
            ) => {
                // Unsigned -> Field: redefine same constant as Field
                SimplifiedTo(dfg.make_constant(constant, dst_typ.clone()))
            }
            (
                Type::Numeric(NumericType::NativeField | NumericType::Unsigned { .. }),
                Type::Numeric(NumericType::Unsigned { bit_size }),
            ) => {
                // Field/Unsigned -> unsigned: truncate
                let integer_modulus = BigUint::from(2u128).pow(*bit_size);
                let constant: BigUint = BigUint::from_bytes_be(&constant.to_be_bytes());
                let truncated = constant % integer_modulus;
                let truncated = FieldElement::from_be_bytes_reduce(&truncated.to_bytes_be());
                SimplifiedTo(dfg.make_constant(truncated, dst_typ.clone()))
            }
            _ => None,
        }
    } else if *dst_typ == dfg.type_of_value(value) {
        SimplifiedTo(value)
    } else {
        None
    }
}

/// The possible return values for Instruction::return_types
pub(crate) enum InstructionResultType {
    /// The result type of this instruction matches that of this operand
    Operand(ValueId),

    /// The result type of this instruction is known to be this type - independent of its operands.
    Known(Type),

    /// The result type of this function is unknown and separate from its operand types.
    /// This occurs for function calls and load operations.
    Unknown,

    /// This instruction does not return any results.
    None,
}

/// These are operations which can exit a basic block
/// ie control flow type operations
///
/// Since our IR needs to be in SSA form, it makes sense
/// to split up instructions like this, as we are sure that these instructions
/// will not be in the list of instructions for a basic block.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum TerminatorInstruction {
    /// Control flow
    ///
    /// Jump If
    ///
    /// If the condition is true: jump to the specified `then_destination`.
    /// Otherwise, jump to the specified `else_destination`.
    JmpIf { condition: ValueId, then_destination: BasicBlockId, else_destination: BasicBlockId },

    /// Unconditional Jump
    ///
    /// Jumps to specified `destination` with `arguments`.
    /// The CallStack here is expected to be used to issue an error when the start range of
    /// a for loop cannot be deduced at compile-time.
    Jmp { destination: BasicBlockId, arguments: Vec<ValueId>, call_stack: CallStack },

    /// Return from the current function with the given return values.
    ///
    /// All finished functions should have exactly 1 return instruction.
    /// Functions with early returns should instead be structured to
    /// unconditionally jump to a single exit block with the return values
    /// as the block arguments. Then the exit block can terminate in a return
    /// instruction returning these values.
    Return { return_values: Vec<ValueId> },
}

impl TerminatorInstruction {
    /// Map each ValueId in this terminator to a new value.
    pub(crate) fn map_values(
        &self,
        mut f: impl FnMut(ValueId) -> ValueId,
    ) -> TerminatorInstruction {
        use TerminatorInstruction::*;
        match self {
            JmpIf { condition, then_destination, else_destination } => JmpIf {
                condition: f(*condition),
                then_destination: *then_destination,
                else_destination: *else_destination,
            },
            Jmp { destination, arguments, call_stack } => Jmp {
                destination: *destination,
                arguments: vecmap(arguments, |value| f(*value)),
                call_stack: call_stack.clone(),
            },
            Return { return_values } => {
                Return { return_values: vecmap(return_values, |value| f(*value)) }
            }
        }
    }

    /// Mutate each ValueId to a new ValueId using the given mapping function
    pub(crate) fn mutate_values(&mut self, mut f: impl FnMut(ValueId) -> ValueId) {
        use TerminatorInstruction::*;
        match self {
            JmpIf { condition, .. } => {
                *condition = f(*condition);
            }
            Jmp { arguments, .. } => {
                for argument in arguments {
                    *argument = f(*argument);
                }
            }
            Return { return_values } => {
                for return_value in return_values {
                    *return_value = f(*return_value);
                }
            }
        }
    }

    /// Apply a function to each value
    pub(crate) fn for_each_value<T>(&self, mut f: impl FnMut(ValueId) -> T) {
        use TerminatorInstruction::*;
        match self {
            JmpIf { condition, .. } => {
                f(*condition);
            }
            Jmp { arguments, .. } => {
                for argument in arguments {
                    f(*argument);
                }
            }
            Return { return_values } => {
                for return_value in return_values {
                    f(*return_value);
                }
            }
        }
    }

    /// Mutate each BlockId to a new BlockId specified by the given mapping function.
    pub(crate) fn mutate_blocks(&mut self, mut f: impl FnMut(BasicBlockId) -> BasicBlockId) {
        use TerminatorInstruction::*;
        match self {
            JmpIf { then_destination, else_destination, .. } => {
                *then_destination = f(*then_destination);
                *else_destination = f(*else_destination);
            }
            Jmp { destination, .. } => {
                *destination = f(*destination);
            }
            Return { .. } => (),
        }
    }
}

/// A binary instruction in the IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Binary {
    /// Left hand side of the binary operation
    pub(crate) lhs: ValueId,
    /// Right hand side of the binary operation
    pub(crate) rhs: ValueId,
    /// The binary operation to apply
    pub(crate) operator: BinaryOp,
}

impl Binary {
    /// The type of this Binary instruction's result
    pub(crate) fn result_type(&self) -> InstructionResultType {
        match self.operator {
            BinaryOp::Eq | BinaryOp::Lt => InstructionResultType::Known(Type::bool()),
            _ => InstructionResultType::Operand(self.lhs),
        }
    }

    /// Try to simplify this binary instruction, returning the new value if possible.
    fn simplify(&self, dfg: &mut DataFlowGraph) -> SimplifyResult {
        let lhs = dfg.get_numeric_constant(self.lhs);
        let rhs = dfg.get_numeric_constant(self.rhs);
        let operand_type = dfg.type_of_value(self.lhs);

        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            // If the rhs of a division is zero, attempting to evaluate the divison will cause a compiler panic.
            // Thus, we do not evaluate this divison as we want to avoid triggering a panic,
            // and division by zero should be handled by laying down constraints during ACIR generation.
            if matches!(self.operator, BinaryOp::Div | BinaryOp::Mod) && rhs == FieldElement::zero()
            {
                return SimplifyResult::None;
            }
            return match self.eval_constants(dfg, lhs, rhs, operand_type) {
                Some(value) => SimplifyResult::SimplifiedTo(value),
                None => SimplifyResult::None,
            };
        }

        let lhs_is_zero = lhs.map_or(false, |lhs| lhs.is_zero());
        let rhs_is_zero = rhs.map_or(false, |rhs| rhs.is_zero());

        let lhs_is_one = lhs.map_or(false, |lhs| lhs.is_one());
        let rhs_is_one = rhs.map_or(false, |rhs| rhs.is_one());

        match self.operator {
            BinaryOp::Add => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Sub => {
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Mul => {
                if lhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if lhs_is_zero || rhs_is_zero {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::Div => {
                if rhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Mod => {
                if rhs_is_one {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::Eq => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let one = dfg.make_constant(FieldElement::one(), Type::bool());
                    return SimplifyResult::SimplifiedTo(one);
                }
                if operand_type == Type::bool() {
                    // Simplify forms of `(boolean == true)` into `boolean`
                    if lhs_is_one {
                        return SimplifyResult::SimplifiedTo(self.rhs);
                    }
                    if rhs_is_one {
                        return SimplifyResult::SimplifiedTo(self.lhs);
                    }
                    // Simplify forms of `(boolean == false)` into `!boolean`
                    if lhs_is_zero {
                        return SimplifyResult::SimplifiedToInstruction(Instruction::Not(self.rhs));
                    }
                    if rhs_is_zero {
                        return SimplifyResult::SimplifiedToInstruction(Instruction::Not(self.lhs));
                    }
                }
            }
            BinaryOp::Lt => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if operand_type.is_unsigned() && rhs_is_zero {
                    // Unsigned values cannot be less than zero.
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::And => {
                if lhs_is_zero || rhs_is_zero {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if operand_type == Type::bool() {
                    // Boolean AND is equivalent to multiplication, which is a cheaper operation.
                    let instruction = Instruction::binary(BinaryOp::Mul, self.lhs, self.rhs);
                    return SimplifyResult::SimplifiedToInstruction(instruction);
                }
            }
            BinaryOp::Or => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Xor => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
        }
        SimplifyResult::None
    }

    /// Evaluate the two constants with the operation specified by self.operator.
    /// Pushes the resulting value to the given DataFlowGraph's constants and returns it.
    fn eval_constants(
        &self,
        dfg: &mut DataFlowGraph,
        lhs: FieldElement,
        rhs: FieldElement,
        mut operand_type: Type,
    ) -> Option<Id<Value>> {
        let value = match &operand_type {
            Type::Numeric(NumericType::NativeField) => {
                self.operator.get_field_function()?(lhs, rhs)
            }
            Type::Numeric(NumericType::Unsigned { bit_size }) => {
                let function = self.operator.get_u128_function();

                let lhs = truncate(lhs.try_into_u128()?, *bit_size);
                let rhs = truncate(rhs.try_into_u128()?, *bit_size);

                // The divisor is being truncated into the type of the operand, which can potentially
                // lead to the rhs being zero.
                // If the rhs of a division is zero, attempting to evaluate the divison will cause a compiler panic.
                // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
                // and the operation should be handled by ACIR generation.
                if matches!(self.operator, BinaryOp::Div) && rhs == 0 {
                    return None;
                }

                let result = function(lhs, rhs);
                truncate(result, *bit_size).into()
            }
            _ => return None,
        };

        if matches!(self.operator, BinaryOp::Eq | BinaryOp::Lt) {
            operand_type = Type::bool();
        }

        Some(dfg.make_constant(value, operand_type))
    }
}

fn truncate(int: u128, bit_size: u32) -> u128 {
    let max = 2u128.pow(bit_size);
    int % max
}

impl BinaryOp {
    fn get_field_function(self) -> Option<fn(FieldElement, FieldElement) -> FieldElement> {
        match self {
            BinaryOp::Add => Some(std::ops::Add::add),
            BinaryOp::Sub => Some(std::ops::Sub::sub),
            BinaryOp::Mul => Some(std::ops::Mul::mul),
            BinaryOp::Div => Some(std::ops::Div::div),
            BinaryOp::Eq => Some(|x, y| (x == y).into()),
            BinaryOp::Lt => Some(|x, y| (x < y).into()),
            // Bitwise operators are unsupported for Fields
            BinaryOp::Mod => None,
            BinaryOp::And => None,
            BinaryOp::Or => None,
            BinaryOp::Xor => None,
        }
    }

    fn get_u128_function(self) -> fn(u128, u128) -> u128 {
        match self {
            BinaryOp::Add => u128::wrapping_add,
            BinaryOp::Sub => u128::wrapping_sub,
            BinaryOp::Mul => u128::wrapping_mul,
            BinaryOp::Div => u128::wrapping_div,
            BinaryOp::Mod => u128::wrapping_rem,
            BinaryOp::And => |x, y| x & y,
            BinaryOp::Or => |x, y| x | y,
            BinaryOp::Xor => |x, y| x ^ y,
            BinaryOp::Eq => |x, y| (x == y) as u128,
            BinaryOp::Lt => |x, y| (x < y) as u128,
        }
    }
}

/// Binary Operations allowed in the IR.
/// Aside from the comparison operators (Eq and Lt), all operators
/// will return the same type as their operands.
/// The operand types must match for all binary operators.
/// All binary operators are also only for numeric types. To implement
/// e.g. equality for a compound type like a struct, one must add a
/// separate Eq operation for each field and combine them later with And.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub(crate) enum BinaryOp {
    /// Addition of lhs + rhs.
    Add,
    /// Subtraction of lhs - rhs.
    Sub,
    /// Multiplication of lhs * rhs.
    Mul,
    /// Division of lhs / rhs.
    Div,
    /// Modulus of lhs % rhs.
    Mod,
    /// Checks whether two types are equal.
    /// Returns true if the types were equal and
    /// false otherwise.
    Eq,
    /// Checks whether the lhs is less than the rhs.
    /// All other comparison operators should be translated
    /// to less than. For example (a > b) = (b < a) = !(a >= b) = !(b <= a).
    /// The result will always be a u1.
    Lt,
    /// Bitwise and (&)
    And,
    /// Bitwise or (|)
    Or,
    /// Bitwise xor (^)
    Xor,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Eq => write!(f, "eq"),
            BinaryOp::Mod => write!(f, "mod"),
            BinaryOp::Lt => write!(f, "lt"),
            BinaryOp::And => write!(f, "and"),
            BinaryOp::Or => write!(f, "or"),
            BinaryOp::Xor => write!(f, "xor"),
        }
    }
}

/// Contains the result to Instruction::simplify, specifying how the instruction
/// should be simplified.
pub(crate) enum SimplifyResult {
    /// Replace this function's result with the given value
    SimplifiedTo(ValueId),

    /// Replace this function's results with the given values
    /// Used for when there are multiple return values from
    /// a function such as a tuple
    SimplifiedToMultiple(Vec<ValueId>),

    /// Replace this function with an simpler but equivalent instruction.
    SimplifiedToInstruction(Instruction),

    /// Remove the instruction, it is unnecessary
    Remove,

    /// Instruction could not be simplified
    None,
}

impl SimplifyResult {
    pub(crate) fn instruction(self) -> Option<Instruction> {
        match self {
            SimplifyResult::SimplifiedToInstruction(instruction) => Some(instruction),
            _ => None,
        }
    }
}
