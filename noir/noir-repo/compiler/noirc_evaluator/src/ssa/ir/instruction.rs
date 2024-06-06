use std::hash::{Hash, Hasher};

use acvm::{
    acir::AcirField,
    acir::{
        circuit::{ErrorSelector, STRING_ERROR_SELECTOR},
        BlackBoxFunc,
    },
    FieldElement,
};
use fxhash::FxHasher;
use iter_extended::vecmap;
use noirc_frontend::hir_def::types::Type as HirType;

use crate::ssa::opt::flatten_cfg::value_merger::ValueMerger;

use super::{
    basic_block::BasicBlockId,
    dfg::{CallStack, DataFlowGraph},
    map::Id,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

mod binary;
mod call;
mod cast;
mod constrain;

pub(crate) use binary::{Binary, BinaryOp};
use call::simplify_call;
use cast::simplify_cast;
use constrain::decompose_constrain;

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
    ArrayLen,
    AsSlice,
    AssertConstant,
    SlicePushBack,
    SlicePushFront,
    SlicePopBack,
    SlicePopFront,
    SliceInsert,
    SliceRemove,
    ApplyRangeConstraint,
    StrAsBytes,
    ToBits(Endian),
    ToRadix(Endian),
    BlackBox(BlackBoxFunc),
    FromField,
    AsField,
    AsWitness,
    IsUnconstrained,
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::ArrayLen => write!(f, "array_len"),
            Intrinsic::AsSlice => write!(f, "as_slice"),
            Intrinsic::AssertConstant => write!(f, "assert_constant"),
            Intrinsic::SlicePushBack => write!(f, "slice_push_back"),
            Intrinsic::SlicePushFront => write!(f, "slice_push_front"),
            Intrinsic::SlicePopBack => write!(f, "slice_pop_back"),
            Intrinsic::SlicePopFront => write!(f, "slice_pop_front"),
            Intrinsic::SliceInsert => write!(f, "slice_insert"),
            Intrinsic::SliceRemove => write!(f, "slice_remove"),
            Intrinsic::StrAsBytes => write!(f, "str_as_bytes"),
            Intrinsic::ApplyRangeConstraint => write!(f, "apply_range_constraint"),
            Intrinsic::ToBits(Endian::Big) => write!(f, "to_be_bits"),
            Intrinsic::ToBits(Endian::Little) => write!(f, "to_le_bits"),
            Intrinsic::ToRadix(Endian::Big) => write!(f, "to_be_radix"),
            Intrinsic::ToRadix(Endian::Little) => write!(f, "to_le_radix"),
            Intrinsic::BlackBox(function) => write!(f, "{function}"),
            Intrinsic::FromField => write!(f, "from_field"),
            Intrinsic::AsField => write!(f, "as_field"),
            Intrinsic::AsWitness => write!(f, "as_witness"),
            Intrinsic::IsUnconstrained => write!(f, "is_unconstrained"),
        }
    }
}

impl Intrinsic {
    /// Returns whether the `Intrinsic` has side effects.
    ///
    /// If there are no side effects then the `Intrinsic` can be removed if the result is unused.
    pub(crate) fn has_side_effects(&self) -> bool {
        match self {
            Intrinsic::AssertConstant | Intrinsic::ApplyRangeConstraint | Intrinsic::AsWitness => {
                true
            }

            // These apply a constraint that the input must fit into a specified number of limbs.
            Intrinsic::ToBits(_) | Intrinsic::ToRadix(_) => true,

            Intrinsic::ArrayLen
            | Intrinsic::AsSlice
            | Intrinsic::SlicePushBack
            | Intrinsic::SlicePushFront
            | Intrinsic::SlicePopBack
            | Intrinsic::SlicePopFront
            | Intrinsic::SliceInsert
            | Intrinsic::SliceRemove
            | Intrinsic::StrAsBytes
            | Intrinsic::FromField
            | Intrinsic::AsField
            | Intrinsic::IsUnconstrained => false,

            // Some black box functions have side-effects
            Intrinsic::BlackBox(func) => matches!(
                func,
                BlackBoxFunc::RecursiveAggregation
                    | BlackBoxFunc::MultiScalarMul
                    | BlackBoxFunc::EmbeddedCurveAdd
            ),
        }
    }

    /// Lookup an Intrinsic by name and return it if found.
    /// If there is no such intrinsic by that name, None is returned.
    pub(crate) fn lookup(name: &str) -> Option<Intrinsic> {
        match name {
            "array_len" => Some(Intrinsic::ArrayLen),
            "as_slice" => Some(Intrinsic::AsSlice),
            "assert_constant" => Some(Intrinsic::AssertConstant),
            "apply_range_constraint" => Some(Intrinsic::ApplyRangeConstraint),
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
            "from_field" => Some(Intrinsic::FromField),
            "as_field" => Some(Intrinsic::AsField),
            "as_witness" => Some(Intrinsic::AsWitness),
            "is_unconstrained" => Some(Intrinsic::IsUnconstrained),
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
    Constrain(ValueId, ValueId, Option<ConstrainError>),

    /// Range constrain `value` to `max_bit_size`
    RangeCheck { value: ValueId, max_bit_size: u32, assert_message: Option<String> },

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
    /// to those in the given array. This will not modify the original array unless `mutable` is
    /// set. This flag is off by default and only enabled when optimizations determine it is safe.
    ArraySet { array: ValueId, index: ValueId, value: ValueId, mutable: bool },

    /// An instruction to increment the reference count of a value.
    ///
    /// This currently only has an effect in Brillig code where array sharing and copy on write is
    /// implemented via reference counting. In ACIR code this is done with im::Vector and these
    /// IncrementRc instructions are ignored.
    IncrementRc { value: ValueId },

    /// An instruction to decrement the reference count of a value.
    ///
    /// This currently only has an effect in Brillig code where array sharing and copy on write is
    /// implemented via reference counting. In ACIR code this is done with im::Vector and these
    /// DecrementRc instructions are ignored.
    DecrementRc { value: ValueId },

    /// Merge two values returned from opposite branches of a conditional into one.
    IfElse {
        then_condition: ValueId,
        then_value: ValueId,
        else_condition: ValueId,
        else_value: ValueId,
    },
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
            Instruction::Not(value)
            | Instruction::Truncate { value, .. }
            | Instruction::ArraySet { array: value, .. }
            | Instruction::IfElse { then_value: value, .. } => {
                InstructionResultType::Operand(*value)
            }
            Instruction::Constrain(..)
            | Instruction::Store { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::RangeCheck { .. }
            | Instruction::EnableSideEffects { .. } => InstructionResultType::None,
            Instruction::Allocate { .. }
            | Instruction::Load { .. }
            | Instruction::ArrayGet { .. }
            | Instruction::Call { .. } => InstructionResultType::Unknown,
        }
    }

    /// True if this instruction requires specifying the control type variables when
    /// inserting this instruction into a DataFlowGraph.
    pub(crate) fn requires_ctrl_typevars(&self) -> bool {
        matches!(self.result_type(), InstructionResultType::Unknown)
    }

    /// Indicates if the instruction can be safely replaced with the results of another instruction with the same inputs.
    pub(crate) fn can_be_deduplicated(&self, dfg: &DataFlowGraph) -> bool {
        use Instruction::*;

        match self {
            // These either have side-effects or interact with memory
            Constrain(..)
            | EnableSideEffects { .. }
            | Allocate
            | Load { .. }
            | Store { .. }
            | IncrementRc { .. }
            | DecrementRc { .. }
            | RangeCheck { .. } => false,

            Call { func, .. } => match dfg[*func] {
                Value::Intrinsic(intrinsic) => !intrinsic.has_side_effects(),
                _ => false,
            },

            // These can have different behavior depending on the EnableSideEffectsIf context.
            // Replacing them with a similar instruction potentially enables replacing an instruction
            // with one that was disabled. See
            // https://github.com/noir-lang/noir/pull/4716#issuecomment-2047846328.
            Binary(_)
            | Cast(_, _)
            | Not(_)
            | Truncate { .. }
            | IfElse { .. }
            | ArrayGet { .. }
            | ArraySet { .. } => !self.requires_acir_gen_predicate(dfg),
        }
    }

    pub(crate) fn can_eliminate_if_unused(&self, dfg: &DataFlowGraph) -> bool {
        use Instruction::*;
        match self {
            Binary(binary) => {
                if matches!(binary.operator, BinaryOp::Div | BinaryOp::Mod) {
                    if let Some(rhs) = dfg.get_numeric_constant(binary.rhs) {
                        rhs != FieldElement::zero()
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            Cast(_, _)
            | Not(_)
            | Truncate { .. }
            | Allocate
            | Load { .. }
            | ArrayGet { .. }
            | IfElse { .. }
            | ArraySet { .. } => true,

            Constrain(..)
            | Store { .. }
            | EnableSideEffects { .. }
            | IncrementRc { .. }
            | DecrementRc { .. }
            | RangeCheck { .. } => false,

            // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
            Call { func, .. } => match dfg[*func] {
                // Explicitly allows removal of unused ec operations, even if they can fail
                Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul))
                | Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::EmbeddedCurveAdd)) => true,
                Value::Intrinsic(intrinsic) => !intrinsic.has_side_effects(),

                // All foreign functions are treated as having side effects.
                // This is because they can be used to pass information
                // from the ACVM to the external world during execution.
                Value::ForeignFunction(_) => false,

                // We must assume that functions contain a side effect as we cannot inspect more deeply.
                Value::Function(_) => false,

                _ => false,
            },
        }
    }

    /// If true the instruction will depends on enable_side_effects context during acir-gen
    fn requires_acir_gen_predicate(&self, dfg: &DataFlowGraph) -> bool {
        match self {
            Instruction::Binary(binary)
                if matches!(binary.operator, BinaryOp::Div | BinaryOp::Mod) =>
            {
                true
            }
            Instruction::EnableSideEffects { .. }
            | Instruction::ArrayGet { .. }
            | Instruction::ArraySet { .. } => true,

            Instruction::Call { func, .. } => match dfg[*func] {
                Value::Function(_) => true,
                Value::Intrinsic(intrinsic) => {
                    matches!(intrinsic, Intrinsic::SliceInsert | Intrinsic::SliceRemove)
                }
                _ => false,
            },
            Instruction::Cast(_, _)
            | Instruction::Binary(_)
            | Instruction::Not(_)
            | Instruction::Truncate { .. }
            | Instruction::Constrain(_, _, _)
            | Instruction::RangeCheck { .. }
            | Instruction::Allocate
            | Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::IfElse { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. } => false,
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
                // Must map the `lhs` and `rhs` first as the value `f` is moved with the closure
                let lhs = f(*lhs);
                let rhs = f(*rhs);
                let assert_message = assert_message.as_ref().map(|error| match error {
                    ConstrainError::UserDefined(selector, payload_values) => {
                        ConstrainError::UserDefined(
                            *selector,
                            payload_values.iter().map(|&value| f(value)).collect(),
                        )
                    }
                    _ => error.clone(),
                });
                Instruction::Constrain(lhs, rhs, assert_message)
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
            Instruction::ArraySet { array, index, value, mutable } => Instruction::ArraySet {
                array: f(*array),
                index: f(*index),
                value: f(*value),
                mutable: *mutable,
            },
            Instruction::IncrementRc { value } => Instruction::IncrementRc { value: f(*value) },
            Instruction::DecrementRc { value } => Instruction::DecrementRc { value: f(*value) },
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                Instruction::RangeCheck {
                    value: f(*value),
                    max_bit_size: *max_bit_size,
                    assert_message: assert_message.clone(),
                }
            }
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                Instruction::IfElse {
                    then_condition: f(*then_condition),
                    then_value: f(*then_value),
                    else_condition: f(*else_condition),
                    else_value: f(*else_value),
                }
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
            | Instruction::Load { address: value } => {
                f(*value);
            }
            Instruction::Constrain(lhs, rhs, assert_error) => {
                f(*lhs);
                f(*rhs);
                if let Some(ConstrainError::UserDefined(_, values)) = assert_error.as_ref() {
                    values.iter().for_each(|&val| {
                        f(val);
                    });
                }
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
            Instruction::ArraySet { array, index, value, mutable: _ } => {
                f(*array);
                f(*index);
                f(*value);
            }
            Instruction::EnableSideEffects { condition } => {
                f(*condition);
            }
            Instruction::IncrementRc { value }
            | Instruction::DecrementRc { value }
            | Instruction::RangeCheck { value, .. } => {
                f(*value);
            }
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                f(*then_condition);
                f(*then_value);
                f(*else_condition);
                f(*else_value);
            }
        }
    }

    /// Try to simplify this instruction. If the instruction can be simplified to a known value,
    /// that value is returned. Otherwise None is returned.
    ///
    /// The `block` parameter indicates the block this new instruction will be inserted into
    /// after this call.
    pub(crate) fn simplify(
        &self,
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: &CallStack,
    ) -> SimplifyResult {
        use SimplifyResult::*;
        match self {
            Instruction::Binary(binary) => binary.simplify(dfg),
            Instruction::Cast(value, typ) => simplify_cast(*value, typ, dfg),
            Instruction::Not(value) => {
                match &dfg[dfg.resolve(*value)] {
                    // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                    // there is no Not on FieldElement, so we'd need to convert between u128. This
                    // would be incorrect however since the extra bits on the field would not be flipped.
                    Value::NumericConstant { constant, typ } if typ.is_unsigned() => {
                        // As we're casting to a `u128`, we need to clear out any upper bits that the NOT fills.
                        let value = !constant.to_u128() % (1 << typ.bit_size());
                        SimplifiedTo(dfg.make_constant(value.into(), typ.clone()))
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
            Instruction::Constrain(lhs, rhs, msg) => {
                let constraints = decompose_constrain(*lhs, *rhs, msg, dfg);
                if constraints.is_empty() {
                    Remove
                } else {
                    SimplifiedToInstructionMultiple(constraints)
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
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                if bit_size == max_bit_size {
                    return SimplifiedTo(*value);
                }
                if let Some((numeric_constant, typ)) = dfg.get_numeric_constant_with_type(*value) {
                    let integer_modulus = 2_u128.pow(*bit_size);
                    let truncated = numeric_constant.to_u128() % integer_modulus;
                    SimplifiedTo(dfg.make_constant(truncated.into(), typ))
                } else if let Value::Instruction { instruction, .. } = &dfg[dfg.resolve(*value)] {
                    match &dfg[*instruction] {
                        Instruction::Truncate { bit_size: src_bit_size, .. } => {
                            // If we're truncating the value to fit into the same or larger bit size then this is a noop.
                            if src_bit_size <= bit_size && src_bit_size <= max_bit_size {
                                SimplifiedTo(*value)
                            } else {
                                None
                            }
                        }

                        Instruction::Binary(Binary {
                            lhs, rhs, operator: BinaryOp::Div, ..
                        }) if dfg.is_constant(*rhs) => {
                            // If we're truncating the result of a division by a constant denominator, we can
                            // reason about the maximum bit size of the result and whether a truncation is necessary.

                            let numerator_type = dfg.type_of_value(*lhs);
                            let max_numerator_bits = numerator_type.bit_size();

                            let divisor = dfg
                                .get_numeric_constant(*rhs)
                                .expect("rhs is checked to be constant.");
                            let divisor_bits = divisor.num_bits();

                            // 2^{max_quotient_bits} = 2^{max_numerator_bits} / 2^{divisor_bits}
                            // => max_quotient_bits = max_numerator_bits - divisor_bits
                            //
                            // In order for the truncation to be a noop, we then require `max_quotient_bits < bit_size`.
                            let max_quotient_bits = max_numerator_bits - divisor_bits;
                            if max_quotient_bits < *bit_size {
                                SimplifiedTo(*value)
                            } else {
                                None
                            }
                        }

                        _ => None,
                    }
                } else {
                    None
                }
            }
            Instruction::Call { func, arguments } => {
                simplify_call(*func, arguments, dfg, block, ctrl_typevars, call_stack)
            }
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
            Instruction::IncrementRc { .. } => None,
            Instruction::DecrementRc { .. } => None,
            Instruction::RangeCheck { value, max_bit_size, .. } => {
                let max_potential_bits = dfg.get_value_max_num_bits(*value);
                if max_potential_bits < *max_bit_size {
                    Remove
                } else {
                    None
                }
            }
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                let typ = dfg.type_of_value(*then_value);

                if let Some(constant) = dfg.get_numeric_constant(*then_condition) {
                    if constant.is_one() {
                        return SimplifiedTo(*then_value);
                    } else if constant.is_zero() {
                        return SimplifiedTo(*else_value);
                    }
                }

                if matches!(&typ, Type::Numeric(_)) {
                    let then_condition = *then_condition;
                    let then_value = *then_value;
                    let else_condition = *else_condition;
                    let else_value = *else_value;

                    let result = ValueMerger::merge_numeric_values(
                        dfg,
                        block,
                        then_condition,
                        else_condition,
                        then_value,
                        else_value,
                    );
                    SimplifiedTo(result)
                } else {
                    None
                }
            }
        }
    }
}

pub(crate) type ErrorType = HirType;

pub(crate) fn error_selector_from_type(typ: &ErrorType) -> ErrorSelector {
    match typ {
        ErrorType::String(_) => STRING_ERROR_SELECTOR,
        _ => {
            let mut hasher = FxHasher::default();
            typ.hash(&mut hasher);
            let hash = hasher.finish();
            assert!(hash != 0, "ICE: Error type {} collides with the string error type", typ);
            ErrorSelector::new(hash)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum ConstrainError {
    // These are errors which have been hardcoded during SSA gen
    Intrinsic(String),
    // These are errors issued by the user
    UserDefined(ErrorSelector, Vec<ValueId>),
}

impl From<String> for ConstrainError {
    fn from(value: String) -> Self {
        ConstrainError::Intrinsic(value)
    }
}

impl From<String> for Box<ConstrainError> {
    fn from(value: String) -> Self {
        Box::new(value.into())
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
    Return { return_values: Vec<ValueId>, call_stack: CallStack },
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
            Return { return_values, call_stack } => Return {
                return_values: vecmap(return_values, |value| f(*value)),
                call_stack: call_stack.clone(),
            },
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
            Return { return_values, .. } => {
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
            Return { return_values, .. } => {
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

    /// Replace this function with a set of simpler but equivalent instructions.
    /// This is currently only to be used for [`Instruction::Constrain`].
    SimplifiedToInstructionMultiple(Vec<Instruction>),

    /// Remove the instruction, it is unnecessary
    Remove,

    /// Instruction could not be simplified
    None,
}

impl SimplifyResult {
    pub(crate) fn instructions(self) -> Option<Vec<Instruction>> {
        match self {
            SimplifyResult::SimplifiedToInstruction(instruction) => Some(vec![instruction]),
            SimplifyResult::SimplifiedToInstructionMultiple(instructions) => Some(instructions),
            _ => None,
        }
    }
}
