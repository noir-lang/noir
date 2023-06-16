use std::rc::Rc;

use acvm::{acir::BlackBoxFunc, FieldElement};
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::ssa_refactor::ir::types::NumericType;

use super::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    map::Id,
    types::Type,
    value::{Value, ValueId},
};

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
    Println,
    ToBits(Endian),
    ToRadix(Endian),
    BlackBox(BlackBoxFunc),
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::Println => write!(f, "println"),
            Intrinsic::Sort => write!(f, "arraysort"),
            Intrinsic::ToBits(Endian::Big) => write!(f, "to_be_bits"),
            Intrinsic::ToBits(Endian::Little) => write!(f, "to_le_bits"),
            Intrinsic::ToRadix(Endian::Big) => write!(f, "to_be_radix"),
            Intrinsic::ToRadix(Endian::Little) => write!(f, "to_le_radix"),
            Intrinsic::BlackBox(function) => write!(f, "{function}"),
        }
    }
}

impl Intrinsic {
    /// Lookup an Intrinsic by name and return it if found.
    /// If there is no such intrinsic by that name, None is returned.
    pub(crate) fn lookup(name: &str) -> Option<Intrinsic> {
        match name {
            "println" => Some(Intrinsic::Println),
            "arraysort" => Some(Intrinsic::Sort),
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

    /// Constrains a value to be equal to true
    Constrain(ValueId),

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
    ArraySet { array: ValueId, index: ValueId, value: ValueId },
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
            Instruction::Constrain(_)
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
            Instruction::Constrain(value) => Instruction::Constrain(f(*value)),
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
            Instruction::ArraySet { array, index, value } => {
                Instruction::ArraySet { array: f(*array), index: f(*index), value: f(*value) }
            }
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
            | Instruction::Constrain(value)
            | Instruction::Load { address: value } => {
                f(*value);
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
            Instruction::ArraySet { array, index, value } => {
                f(*array);
                f(*index);
                f(*value);
            }
            Instruction::EnableSideEffects { condition } => {
                f(*condition);
            }
        }
    }

    /// Try to simplify this instruction. If the instruction can be simplified to a known value,
    /// that value is returned. Otherwise None is returned.
    pub(crate) fn simplify(&self, dfg: &mut DataFlowGraph) -> SimplifyResult {
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
            Instruction::Constrain(value) => {
                if let Some(constant) = dfg.get_numeric_constant(*value) {
                    if constant.is_one() {
                        return Remove;
                    }
                }
                None
            }
            Instruction::ArrayGet { array, index } => {
                let array = dfg.get_array_constant(*array);
                let index = dfg.get_numeric_constant(*index);

                if let (Some((array, _)), Some(index)) = (array, index) {
                    let index =
                        index.try_to_u64().expect("Expected array index to fit in u64") as usize;
                    assert!(index < array.len());
                    SimplifiedTo(array[index])
                } else {
                    None
                }
            }
            Instruction::ArraySet { array, index, value } => {
                let array = dfg.get_array_constant(*array);
                let index = dfg.get_numeric_constant(*index);

                if let (Some((array, element_type)), Some(index)) = (array, index) {
                    let index =
                        index.try_to_u64().expect("Expected array index to fit in u64") as usize;
                    assert!(index < array.len());
                    SimplifiedTo(dfg.make_array(array.update(index, *value), element_type))
                } else {
                    None
                }
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
            Instruction::Call { func, arguments } => simplify_call(*func, arguments, dfg),
            Instruction::Allocate { .. } => None,
            Instruction::Load { .. } => None,
            Instruction::Store { .. } => None,
            Instruction::EnableSideEffects { .. } => None,
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

/// Try to simplify this call instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
fn simplify_call(func: ValueId, arguments: &[ValueId], dfg: &mut DataFlowGraph) -> SimplifyResult {
    use SimplifyResult::*;
    let intrinsic = match &dfg[func] {
        Value::Intrinsic(intrinsic) => *intrinsic,
        _ => return None,
    };
    let constant_args: Option<Vec<_>> =
        arguments.iter().map(|value_id| dfg.get_numeric_constant(*value_id)).collect();
    let constant_args = match constant_args {
        Some(constant_args) => constant_args,
        Option::None => return None,
    };
    match intrinsic {
        Intrinsic::ToBits(endian) => {
            let field = constant_args[0];
            let limb_count = constant_args[1].to_u128() as u32;
            SimplifiedTo(constant_to_radix(endian, field, 2, limb_count, dfg))
        }
        Intrinsic::ToRadix(endian) => {
            let field = constant_args[0];
            let radix = constant_args[1].to_u128() as u32;
            let limb_count = constant_args[1].to_u128() as u32;
            SimplifiedTo(constant_to_radix(endian, field, radix, limb_count, dfg))
        }
        Intrinsic::BlackBox(_) | Intrinsic::Println | Intrinsic::Sort => None,
    }
}

/// Returns a Value::Array of constants corresponding to the limbs of the radix decomposition.
fn constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
    dfg: &mut DataFlowGraph,
) -> ValueId {
    let bit_size = u32::BITS - (radix - 1).leading_zeros();
    let radix_big = BigUint::from(radix);
    assert_eq!(BigUint::from(2u128).pow(bit_size), radix_big, "ICE: Radix must be a power of 2");
    let big_integer = BigUint::from_bytes_be(&field.to_be_bytes());

    // Decompose the integer into its radix digits in little endian form.
    let decomposed_integer = big_integer.to_radix_le(radix);
    let mut limbs = vecmap(0..limb_count, |i| match decomposed_integer.get(i as usize) {
        Some(digit) => FieldElement::from_be_bytes_reduce(&[*digit]),
        None => FieldElement::zero(),
    });
    if endian == Endian::Big {
        limbs.reverse();
    }

    // For legacy reasons (see #617) the to_radix interface supports 256 bits even though
    // FieldElement::max_num_bits() is only 254 bits. Any limbs beyond the specified count
    // become zero padding.
    let max_decomposable_bits: u32 = 256;
    let limb_count_with_padding = max_decomposable_bits / bit_size;
    while limbs.len() < limb_count_with_padding as usize {
        limbs.push(FieldElement::zero());
    }
    let result_constants =
        limbs.into_iter().map(|limb| dfg.make_constant(limb, Type::unsigned(bit_size))).collect();
    dfg.make_array(result_constants, Rc::new(vec![Type::unsigned(bit_size)]))
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
    /// Jumps to specified `destination` with `arguments`
    Jmp { destination: BasicBlockId, arguments: Vec<ValueId> },

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
            Jmp { destination, arguments } => {
                Jmp { destination: *destination, arguments: vecmap(arguments, |value| f(*value)) }
            }
            Return { return_values } => {
                Return { return_values: vecmap(return_values, |value| f(*value)) }
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
            }
            BinaryOp::Lt => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::And => {
                if lhs_is_zero || rhs_is_zero {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::Or => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Xor => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::Shl => {
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Shr => {
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
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
        let value = match self.operator {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Eq => {
                operand_type = Type::bool();
                (lhs == rhs).into()
            }
            BinaryOp::Lt => {
                operand_type = Type::bool();
                (lhs < rhs).into()
            }

            // The rest of the operators we must try to convert to u128 first
            BinaryOp::Mod => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::And => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Or => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Xor => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Shl => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Shr => self.eval_constant_u128_operations(lhs, rhs)?,
        };
        Some(dfg.make_constant(value, operand_type))
    }

    /// Try to evaluate the given operands as u128s for operators that are only valid on u128s,
    /// like the bitwise operators and modulus.
    fn eval_constant_u128_operations(
        &self,
        lhs: FieldElement,
        rhs: FieldElement,
    ) -> Option<FieldElement> {
        let lhs = lhs.try_into_u128()?;
        let rhs = rhs.try_into_u128()?;
        match self.operator {
            BinaryOp::Mod => Some((lhs % rhs).into()),
            BinaryOp::And => Some((lhs & rhs).into()),
            BinaryOp::Or => Some((lhs | rhs).into()),
            BinaryOp::Shr => Some((lhs >> rhs).into()),
            // Check for overflow and return None if anything does overflow
            BinaryOp::Shl => {
                let rhs = rhs.try_into().ok()?;
                lhs.checked_shl(rhs).map(Into::into)
            }

            // Converting a field xor to a u128 xor would be incorrect since we wouldn't have the
            // extra bits of the field. So we don't optimize it here.
            BinaryOp::Xor => None,

            op @ (BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Eq
            | BinaryOp::Lt) => panic!(
                "eval_constant_u128_operations invalid for {op:?} use eval_constants instead"
            ),
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
    /// Shift lhs left by rhs bits (<<)
    Shl,
    /// Shift lhs right by rhs bits (>>)
    Shr,
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
            BinaryOp::Shl => write!(f, "shl"),
            BinaryOp::Shr => write!(f, "shr"),
        }
    }
}

/// Contains the result to Instruction::simplify, specifying how the instruction
/// should be simplified.
pub(crate) enum SimplifyResult {
    /// Replace this function's result with the given value
    SimplifiedTo(ValueId),

    /// Remove the instruction, it is unnecessary
    Remove,

    /// Instruction could not be simplified
    None,
}
