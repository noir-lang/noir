use noirc_errors::call_stack::CallStackId;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use acvm::acir::{BlackBoxFunc, circuit::ErrorSelector};
use fxhash::FxHasher64;
use iter_extended::vecmap;
use noirc_frontend::hir_def::types::Type as HirType;

use crate::ssa::opt::pure::Purity;

use super::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    map::Id,
    types::{NumericType, Type},
    value::{Value, ValueId, ValueMapping},
};

pub mod binary;

pub use binary::{Binary, BinaryOp};

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
///   special support for. (LowLevel)
/// - Opcodes which have no function definition in the
///   source code and must be processed by the IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intrinsic {
    ArrayLen,
    ArrayAsStrUnchecked,
    AsSlice,
    AssertConstant,
    StaticAssert,
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
    Hint(Hint),
    AsWitness,
    IsUnconstrained,
    DerivePedersenGenerators,
    FieldLessThan,
    ArrayRefCount,
    SliceRefCount,
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::ArrayLen => write!(f, "array_len"),
            Intrinsic::ArrayAsStrUnchecked => write!(f, "array_as_str_unchecked"),
            Intrinsic::AsSlice => write!(f, "as_slice"),
            Intrinsic::AssertConstant => write!(f, "assert_constant"),
            Intrinsic::StaticAssert => write!(f, "static_assert"),
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
            Intrinsic::Hint(Hint::BlackBox) => write!(f, "black_box"),
            Intrinsic::AsWitness => write!(f, "as_witness"),
            Intrinsic::IsUnconstrained => write!(f, "is_unconstrained"),
            Intrinsic::DerivePedersenGenerators => write!(f, "derive_pedersen_generators"),
            Intrinsic::FieldLessThan => write!(f, "field_less_than"),
            Intrinsic::ArrayRefCount => write!(f, "array_refcount"),
            Intrinsic::SliceRefCount => write!(f, "slice_refcount"),
        }
    }
}

impl Intrinsic {
    /// Returns whether the `Intrinsic` has side effects.
    ///
    /// If there are no side effects then the `Intrinsic` can be removed if the result is unused.
    ///
    /// An example of a side effect is increasing the reference count of an array, but functions
    /// which can fail due to implicit constraints are also considered to have a side effect.
    pub(crate) fn has_side_effects(&self) -> bool {
        match self {
            Intrinsic::AssertConstant
            | Intrinsic::StaticAssert
            | Intrinsic::ApplyRangeConstraint
            // Array & slice ref counts are treated as having side effects since they operate
            // on hidden variables on otherwise identical array values.
            | Intrinsic::ArrayRefCount
            | Intrinsic::SliceRefCount
            | Intrinsic::AsWitness => true,

            // These apply a constraint that the input must fit into a specified number of limbs.
            Intrinsic::ToBits(_) | Intrinsic::ToRadix(_) => true,

            // These imply a check that the slice is non-empty and should fail otherwise.
            Intrinsic::SlicePopBack | Intrinsic::SlicePopFront | Intrinsic::SliceRemove => true,

            Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::AsSlice
            | Intrinsic::SlicePushBack
            | Intrinsic::SlicePushFront
            | Intrinsic::SliceInsert
            | Intrinsic::StrAsBytes
            | Intrinsic::IsUnconstrained
            | Intrinsic::DerivePedersenGenerators
            | Intrinsic::FieldLessThan => false,

            // Treat the black_box hint as-if it could potentially have side effects.
            Intrinsic::Hint(Hint::BlackBox) => true,

            // Some black box functions have side-effects
            Intrinsic::BlackBox(func) => func.has_side_effects(),
        }
    }

    pub(crate) fn purity(&self) -> Purity {
        match self {
            // These apply a constraint in the form of ACIR opcodes, but they can be deduplicated
            // if the inputs are the same. If they depend on a side effect variable (e.g. because
            // they were in an if-then-else) then `handle_instruction_side_effects` in `flatten_cfg`
            // will have attached the condition variable to their inputs directly, so they don't
            // directly depend on the corresponding `enable_side_effect` instruction any more.
            // However, to conform with the expectations of `Instruction::can_be_deduplicated` and
            // `constant_folding` we only use this information if the caller shows interest in it.
            Intrinsic::ToBits(_) | Intrinsic::ToRadix(_) => Purity::PureWithPredicate,
            Intrinsic::BlackBox(func) if func.has_side_effects() => Purity::PureWithPredicate,

            // Operations that remove items from a slice don't modify the slice, they just assert it's non-empty.
            Intrinsic::SlicePopBack | Intrinsic::SlicePopFront | Intrinsic::SliceRemove => {
                Purity::PureWithPredicate
            }

            Intrinsic::AssertConstant
            | Intrinsic::StaticAssert
            | Intrinsic::ApplyRangeConstraint
            | Intrinsic::AsWitness => Purity::PureWithPredicate,

            _ if self.has_side_effects() => Purity::Impure,
            _ => Purity::Pure,
        }
    }

    /// Lookup an Intrinsic by name and return it if found.
    /// If there is no such intrinsic by that name, None is returned.
    pub(crate) fn lookup(name: &str) -> Option<Intrinsic> {
        match name {
            "array_len" => Some(Intrinsic::ArrayLen),
            "array_as_str_unchecked" => Some(Intrinsic::ArrayAsStrUnchecked),
            "as_slice" => Some(Intrinsic::AsSlice),
            "assert_constant" => Some(Intrinsic::AssertConstant),
            "static_assert" => Some(Intrinsic::StaticAssert),
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
            "as_witness" => Some(Intrinsic::AsWitness),
            "is_unconstrained" => Some(Intrinsic::IsUnconstrained),
            "derive_pedersen_generators" => Some(Intrinsic::DerivePedersenGenerators),
            "field_less_than" => Some(Intrinsic::FieldLessThan),
            "black_box" => Some(Intrinsic::Hint(Hint::BlackBox)),
            "array_refcount" => Some(Intrinsic::ArrayRefCount),
            "slice_refcount" => Some(Intrinsic::SliceRefCount),

            other => BlackBoxFunc::lookup(other).map(Intrinsic::BlackBox),
        }
    }
}

/// The endian-ness of bits when encoding values as bits in e.g. ToBits or ToRadix
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endian {
    Big,
    Little,
}

/// Compiler hints.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hint {
    /// Hint to the compiler to treat the call as having potential side effects,
    /// so that the value passed to it can survive SSA passes without being
    /// simplified out completely. This facilitates testing and reproducing
    /// runtime behavior with constants.
    BlackBox,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
/// Instructions are used to perform tasks.
/// The instructions that the IR is able to specify are listed below.
pub enum Instruction {
    /// Binary Operations like +, -, *, /, ==, !=
    Binary(Binary),

    /// Converts `Value` into the given `NumericType`
    ///
    /// This operation only changes the type of the value, it does not change the value itself.
    /// It is expected that the value can fit into the target type.
    /// For instance a value of type `u32` casted to `u8` must already fit into 8 bits
    /// A value of type `i8` cannot be casted to 'i16' since the value would need to include the sign bit (which is the MSB)
    /// Ssa code-gen must ensure that the necessary truncation or sign extension is performed when emitting a Cast instruction.
    Cast(ValueId, NumericType),

    /// Computes a bit wise not
    Not(ValueId),

    /// Truncates `value` to `bit_size`
    Truncate { value: ValueId, bit_size: u32, max_bit_size: u32 },

    /// Constrains two values to be equal to one another.
    Constrain(ValueId, ValueId, Option<ConstrainError>),

    /// Constrains two values to not be equal to one another.
    ConstrainNotEqual(ValueId, ValueId, Option<ConstrainError>),

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
    /// `EnableSideEffectsIf` is encountered, for stating a condition that determines whether
    /// such instructions are allowed to have side-effects.
    ///
    /// For example,
    /// ```text
    /// EnableSideEffectsIf condition0;
    /// code0;
    /// EnableSideEffectsIf condition1;
    /// code1;
    /// ```
    /// - `code0` will have side effects iff `condition0` evaluates to `true`
    /// - `code1` will have side effects iff `condition1` evaluates to `true`
    ///
    /// This instruction is only emitted after the cfg flattening pass, and is used to annotate
    /// instruction regions with a condition that corresponds to their position in the CFG's
    /// if-branching structure.
    EnableSideEffectsIf { condition: ValueId },

    /// Retrieve a value from an array at the given index
    /// `offset` determines whether the index has been shifted by some offset.
    ArrayGet { array: ValueId, index: ValueId, offset: ArrayOffset },

    /// Creates a new array with the new value at the given index. All other elements are identical
    /// to those in the given array. This will not modify the original array unless `mutable` is
    /// set. This flag is off by default and only enabled when optimizations determine it is safe.
    /// `offset` determines whether the index has been shifted by some offset.
    ArraySet { array: ValueId, index: ValueId, value: ValueId, mutable: bool, offset: ArrayOffset },

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
    ///
    /// ```text
    /// if then_condition {
    ///     then_value
    /// } else {   // else_condition = !then_condition
    ///     else_value
    /// }
    /// ```
    IfElse {
        then_condition: ValueId,
        then_value: ValueId,
        else_condition: ValueId,
        else_value: ValueId,
    },

    /// Creates a new array or slice.
    ///
    /// `typ` should be an array or slice type with an element type
    /// matching each of the `elements` values' types.
    MakeArray { elements: im::Vector<ValueId>, typ: Type },

    /// A No-op instruction. These are intended to replace other instructions in a block's
    /// instructions vector without having to move each instruction afterward.
    ///
    /// A No-op has no results and is always removed when Instruction::simplify is called.
    /// When replacing another instruction, the instruction's results should always be mapped to a
    /// new value since they will not be able to refer to their original instruction value any more.
    Noop,
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
            Instruction::Cast(_, typ) => InstructionResultType::Known(Type::Numeric(*typ)),
            Instruction::MakeArray { typ, .. } => InstructionResultType::Known(typ.clone()),
            Instruction::Not(value)
            | Instruction::Truncate { value, .. }
            | Instruction::ArraySet { array: value, .. }
            | Instruction::IfElse { then_value: value, .. } => {
                InstructionResultType::Operand(*value)
            }
            Instruction::Constrain(..)
            | Instruction::ConstrainNotEqual(..)
            | Instruction::Store { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::RangeCheck { .. }
            | Instruction::Noop
            | Instruction::EnableSideEffectsIf { .. } => InstructionResultType::None,
            Instruction::Allocate
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

    /// If true the instruction will depend on `enable_side_effects` context during acir-gen.
    pub(crate) fn requires_acir_gen_predicate(&self, dfg: &DataFlowGraph) -> bool {
        match self {
            Instruction::Binary(binary) => binary.requires_acir_gen_predicate(dfg),

            Instruction::ArrayGet { array, index, offset: _ } => {
                // `ArrayGet`s which read from "known good" indices from an array should not need a predicate.
                !dfg.is_safe_index(*index, *array)
            }

            Instruction::EnableSideEffectsIf { .. } | Instruction::ArraySet { .. } => true,

            Instruction::Call { func, .. } => match dfg[*func] {
                Value::Function(id) => !matches!(dfg.purity_of(id), Some(Purity::Pure)),
                Value::Intrinsic(intrinsic) => {
                    // These utilize `noirc_evaluator::acir::Context::get_flattened_index` internally
                    // which uses the side effects predicate.
                    matches!(intrinsic, Intrinsic::SliceInsert | Intrinsic::SliceRemove)
                }
                _ => false,
            },
            Instruction::Cast(_, _)
            | Instruction::Not(_)
            | Instruction::Truncate { .. }
            | Instruction::ConstrainNotEqual(..)
            | Instruction::Constrain(_, _, _)
            | Instruction::RangeCheck { .. }
            | Instruction::Allocate
            | Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::IfElse { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::Noop
            | Instruction::MakeArray { .. } => false,
        }
    }

    /// Indicates if the instruction has a side effect, ie. it can fail, or it interacts with memory.
    pub(crate) fn has_side_effects(&self, dfg: &DataFlowGraph) -> bool {
        use Instruction::*;

        match self {
            // These either have side-effects or interact with memory
            EnableSideEffectsIf { .. }
            | Allocate
            | Load { .. }
            | Store { .. }
            | IncrementRc { .. }
            | DecrementRc { .. } => true,

            Call { func, .. } => match dfg[*func] {
                Value::Intrinsic(intrinsic) => intrinsic.has_side_effects(),
                // Functions known to be pure have no side effects.
                // `PureWithPredicates` functions may still have side effects.
                Value::Function(function) => dfg.purity_of(function) != Some(Purity::Pure),
                _ => true, // Be conservative and assume other functions can have side effects.
            },

            // These can fail.
            Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => true,

            // This should never be side-effectual
            MakeArray { .. } | Noop => false,

            // Some binary math can overflow or underflow
            Binary(binary) => binary.has_side_effects(),

            // These don't have side effects
            Cast(_, _) | Not(_) | Truncate { .. } | IfElse { .. } => false,

            // `ArrayGet`s which read from "known good" indices from an array have no side effects
            ArrayGet { array, index, offset: _ } => !dfg.is_safe_index(*index, *array),

            // ArraySet has side effects
            ArraySet { .. } => true,
        }
    }

    /// Replaces values present in this instruction with other values according to the given mapping.
    pub(crate) fn replace_values(&mut self, mapping: &ValueMapping) {
        if !mapping.is_empty() {
            self.map_values_mut(|value_id| mapping.get(value_id));
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
            Instruction::Cast(value, typ) => Instruction::Cast(f(*value), *typ),
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
                    ConstrainError::Dynamic(selector, is_string, payload_values) => {
                        ConstrainError::Dynamic(
                            *selector,
                            *is_string,
                            payload_values.iter().map(|&value| f(value)).collect(),
                        )
                    }
                    _ => error.clone(),
                });
                Instruction::Constrain(lhs, rhs, assert_message)
            }
            Instruction::ConstrainNotEqual(lhs, rhs, assert_message) => {
                // Must map the `lhs` and `rhs` first as the value `f` is moved with the closure
                let lhs = f(*lhs);
                let rhs = f(*rhs);
                let assert_message = assert_message.as_ref().map(|error| match error {
                    ConstrainError::Dynamic(selector, is_string, payload_values) => {
                        ConstrainError::Dynamic(
                            *selector,
                            *is_string,
                            payload_values.iter().map(|&value| f(value)).collect(),
                        )
                    }
                    _ => error.clone(),
                });
                Instruction::ConstrainNotEqual(lhs, rhs, assert_message)
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
            Instruction::EnableSideEffectsIf { condition } => {
                Instruction::EnableSideEffectsIf { condition: f(*condition) }
            }
            Instruction::ArrayGet { array, index, offset } => {
                Instruction::ArrayGet { array: f(*array), index: f(*index), offset: *offset }
            }
            Instruction::ArraySet { array, index, value, mutable, offset } => {
                Instruction::ArraySet {
                    array: f(*array),
                    index: f(*index),
                    value: f(*value),
                    mutable: *mutable,
                    offset: *offset,
                }
            }
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
            Instruction::MakeArray { elements, typ } => Instruction::MakeArray {
                elements: elements.iter().copied().map(f).collect(),
                typ: typ.clone(),
            },
            Instruction::Noop => Instruction::Noop,
        }
    }

    /// Maps each ValueId inside this instruction to a new ValueId in place.
    pub(crate) fn map_values_mut(&mut self, mut f: impl FnMut(ValueId) -> ValueId) {
        match self {
            Instruction::Binary(binary) => {
                binary.lhs = f(binary.lhs);
                binary.rhs = f(binary.rhs);
            }
            Instruction::Cast(value, _) => *value = f(*value),
            Instruction::Not(value) => *value = f(*value),
            Instruction::Truncate { value, bit_size: _, max_bit_size: _ } => {
                *value = f(*value);
            }
            Instruction::Constrain(lhs, rhs, assert_message)
            | Instruction::ConstrainNotEqual(lhs, rhs, assert_message) => {
                *lhs = f(*lhs);
                *rhs = f(*rhs);
                if let Some(ConstrainError::Dynamic(_, _, payload_values)) = assert_message {
                    for value in payload_values {
                        *value = f(*value);
                    }
                }
            }
            Instruction::Call { func, arguments } => {
                *func = f(*func);
                for argument in arguments {
                    *argument = f(*argument);
                }
            }
            Instruction::Allocate => (),
            Instruction::Load { address } => *address = f(*address),
            Instruction::Store { address, value } => {
                *address = f(*address);
                *value = f(*value);
            }
            Instruction::EnableSideEffectsIf { condition } => {
                *condition = f(*condition);
            }
            Instruction::ArrayGet { array, index, offset: _ } => {
                *array = f(*array);
                *index = f(*index);
            }
            Instruction::ArraySet { array, index, value, mutable: _, offset: _ } => {
                *array = f(*array);
                *index = f(*index);
                *value = f(*value);
            }
            Instruction::IncrementRc { value } => *value = f(*value),
            Instruction::DecrementRc { value } => {
                *value = f(*value);
            }
            Instruction::RangeCheck { value, max_bit_size: _, assert_message: _ } => {
                *value = f(*value);
            }
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                *then_condition = f(*then_condition);
                *then_value = f(*then_value);
                *else_condition = f(*else_condition);
                *else_value = f(*else_value);
            }
            Instruction::MakeArray { elements, typ: _ } => {
                for element in elements.iter_mut() {
                    *element = f(*element);
                }
            }
            Instruction::Noop => (),
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
            Instruction::Constrain(lhs, rhs, assert_error)
            | Instruction::ConstrainNotEqual(lhs, rhs, assert_error) => {
                f(*lhs);
                f(*rhs);
                if let Some(ConstrainError::Dynamic(_, _, values)) = assert_error.as_ref() {
                    values.iter().for_each(|&val| {
                        f(val);
                    });
                }
            }

            Instruction::Store { address, value } => {
                f(*address);
                f(*value);
            }
            Instruction::Allocate => (),
            Instruction::ArrayGet { array, index, offset: _ } => {
                f(*array);
                f(*index);
            }
            Instruction::ArraySet { array, index, value, mutable: _, offset: _ } => {
                f(*array);
                f(*index);
                f(*value);
            }
            Instruction::EnableSideEffectsIf { condition } => {
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
            Instruction::MakeArray { elements, typ: _ } => {
                for element in elements {
                    f(*element);
                }
            }
            Instruction::Noop => (),
        }
    }
}

/// Determines whether an ArrayGet or ArraySet index has been shifted by a given value.
/// Offsets are set during `crate::ssa::opt::brillig_array_gets` for brillig arrays
/// and vectors with constant indices.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum ArrayOffset {
    None,
    Array,
    Slice,
}

impl ArrayOffset {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Array),
            3 => Some(Self::Slice),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Self::None => 0,
            // Arrays in brillig are represented as [RC, ...items]
            Self::Array => 1,
            // Slices in brillig are represented as [RC, Size, Capacity, ...items]
            Self::Slice => 3,
        }
    }
}

impl Binary {
    pub(crate) fn requires_acir_gen_predicate(&self, dfg: &DataFlowGraph) -> bool {
        match self.operator {
            BinaryOp::Add { unchecked: false }
            | BinaryOp::Sub { unchecked: false }
            | BinaryOp::Mul { unchecked: false } => {
                // Some binary math can overflow or underflow, but this is only the case
                // for unsigned types (here we assume the type of binary.lhs is the same)
                dfg.type_of_value(self.rhs).is_unsigned()
            }
            BinaryOp::Div | BinaryOp::Mod => true,
            BinaryOp::Add { unchecked: true }
            | BinaryOp::Sub { unchecked: true }
            | BinaryOp::Mul { unchecked: true }
            | BinaryOp::Eq
            | BinaryOp::Lt
            | BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Xor
            | BinaryOp::Shl
            | BinaryOp::Shr => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ErrorType {
    String(String),
    Dynamic(HirType),
}

impl ErrorType {
    pub fn selector(&self) -> ErrorSelector {
        let mut hasher = FxHasher64::default();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        ErrorSelector::new(hash)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum ConstrainError {
    // Static string errors are not handled inside the program as data for efficiency reasons.
    StaticString(String),
    // These errors are handled by the program as data.
    // We use a boolean to indicate if the error is a string for printing purposes.
    Dynamic(ErrorSelector, /* is_string */ bool, Vec<ValueId>),
}

impl From<String> for ConstrainError {
    fn from(value: String) -> Self {
        ConstrainError::StaticString(value)
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) enum TerminatorInstruction {
    /// Control flow
    ///
    /// Jump If
    ///
    /// If the condition is true: jump to the specified `then_destination`.
    /// Otherwise, jump to the specified `else_destination`.
    JmpIf {
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
        call_stack: CallStackId,
    },

    /// Unconditional Jump
    ///
    /// Jumps to specified `destination` with `arguments`.
    /// The CallStack here is expected to be used to issue an error when the start range of
    /// a for loop cannot be deduced at compile-time.
    Jmp { destination: BasicBlockId, arguments: Vec<ValueId>, call_stack: CallStackId },

    /// Return from the current function with the given return values.
    ///
    /// All finished functions should have exactly 1 return instruction.
    /// Functions with early returns should instead be structured to
    /// unconditionally jump to a single exit block with the return values
    /// as the block arguments. Then the exit block can terminate in a return
    /// instruction returning these values.
    Return { return_values: Vec<ValueId>, call_stack: CallStackId },

    /// A terminator that will never be reached because an instruction in its block
    /// will always produce an assertion failure.
    Unreachable { call_stack: CallStackId },
}

impl TerminatorInstruction {
    /// Mutate each ValueId to a new ValueId using the given mapping function
    pub(crate) fn map_values_mut(&mut self, mut f: impl FnMut(ValueId) -> ValueId) {
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
            Unreachable { .. } => (),
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
            Unreachable { .. } => (),
        }
    }

    /// Apply a function to each value along with its index
    pub(crate) fn for_eachi_value<T>(&self, mut f: impl FnMut(usize, ValueId) -> T) {
        use TerminatorInstruction::*;
        match self {
            JmpIf { condition, .. } => {
                f(0, *condition);
            }
            Jmp { arguments, .. } => {
                for (index, argument) in arguments.iter().enumerate() {
                    f(index, *argument);
                }
            }
            Return { return_values, .. } => {
                for (index, return_value) in return_values.iter().enumerate() {
                    f(index, *return_value);
                }
            }
            Unreachable { .. } => (),
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
            Return { .. } | Unreachable { .. } => (),
        }
    }

    pub(crate) fn call_stack(&self) -> CallStackId {
        match self {
            TerminatorInstruction::JmpIf { call_stack, .. }
            | TerminatorInstruction::Jmp { call_stack, .. }
            | TerminatorInstruction::Return { call_stack, .. }
            | TerminatorInstruction::Unreachable { call_stack } => *call_stack,
        }
    }

    pub(crate) fn set_call_stack(&mut self, new_call_stack: CallStackId) {
        match self {
            TerminatorInstruction::JmpIf { call_stack, .. }
            | TerminatorInstruction::Jmp { call_stack, .. }
            | TerminatorInstruction::Return { call_stack, .. }
            | TerminatorInstruction::Unreachable { call_stack } => *call_stack = new_call_stack,
        }
    }
}
