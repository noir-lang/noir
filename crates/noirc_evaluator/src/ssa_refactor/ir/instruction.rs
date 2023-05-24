use acvm::{acir::BlackBoxFunc, FieldElement};
use iter_extended::vecmap;

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
    ///
    /// `size` is the size of the region to be allocated by the number of FieldElements it
    /// contains. Note that non-numeric types like Functions and References are counted as 1 field
    /// each.
    Allocate { size: u32 },

    /// Loads a value from memory.
    Load { address: ValueId },

    /// Writes a value to memory.
    Store { address: ValueId, value: ValueId },
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
            Instruction::Cast(_, typ) => InstructionResultType::Known(*typ),
            Instruction::Allocate { .. } => InstructionResultType::Known(Type::Reference),
            Instruction::Not(value) | Instruction::Truncate { value, .. } => {
                InstructionResultType::Operand(*value)
            }
            Instruction::Constrain(_) | Instruction::Store { .. } => InstructionResultType::None,
            Instruction::Load { .. } | Instruction::Call { .. } => InstructionResultType::Unknown,
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
            Instruction::Cast(value, typ) => Instruction::Cast(f(*value), *typ),
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
            Instruction::Allocate { size } => Instruction::Allocate { size: *size },
            Instruction::Load { address } => Instruction::Load { address: f(*address) },
            Instruction::Store { address, value } => {
                Instruction::Store { address: f(*address), value: f(*value) }
            }
        }
    }

    /// Try to simplify this instruction. If the instruction can be simplified to a known value,
    /// that value is returned. Otherwise None is returned.
    pub(crate) fn simplify(&self, dfg: &mut DataFlowGraph) -> SimplifyResult {
        use SimplifyResult::*;
        match self {
            Instruction::Binary(binary) => binary.simplify(dfg),
            Instruction::Cast(value, typ) => {
                match (*typ == dfg.type_of_value(*value)).then_some(*value) {
                    Some(value) => SimplifiedTo(value),
                    _ => None,
                }
            }
            Instruction::Not(value) => {
                match &dfg[*value] {
                    // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                    // there is no Not on FieldElement, so we'd need to convert between u128. This
                    // would be incorrect however since the extra bits on the field would not be flipped.
                    Value::NumericConstant { constant, typ } if *typ == Type::bool() => {
                        let value = dfg[*constant].value().is_zero() as u128;
                        SimplifiedTo(dfg.make_constant(value.into(), Type::bool()))
                    }
                    Value::Instruction { instruction, .. } => {
                        // !!v => v
                        match &dfg[*instruction] {
                            Instruction::Not(value) => SimplifiedTo(*value),
                            _ => None,
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
            Instruction::Truncate { .. } => None,
            Instruction::Call { .. } => None,
            Instruction::Allocate { .. } => None,
            Instruction::Load { .. } => None,
            Instruction::Store { .. } => None,
        }
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
                if self.lhs == self.rhs {
                    let one = dfg.make_constant(FieldElement::one(), Type::bool());
                    return SimplifyResult::SimplifiedTo(one);
                }
            }
            BinaryOp::Lt => {
                if self.lhs == self.rhs {
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
                if self.lhs == self.rhs {
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
        operand_type: Type,
    ) -> Option<Id<Value>> {
        let value = match self.operator {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Eq => (lhs == rhs).into(),
            BinaryOp::Lt => (lhs < rhs).into(),

            // The rest of the operators we must try to convert to u128 first
            BinaryOp::Mod => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::And => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Or => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Xor => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Shl => self.eval_constant_u128_operations(lhs, rhs)?,
            BinaryOp::Shr => self.eval_constant_u128_operations(lhs, rhs)?,
        };
        // TODO: Keep original type of constant
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
