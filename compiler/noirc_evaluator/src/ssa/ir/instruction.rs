use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use acvm::{
    acir::AcirField,
    acir::{circuit::ErrorSelector, BlackBoxFunc},
    FieldElement,
};
use fxhash::FxHasher64;
use iter_extended::vecmap;
use noirc_frontend::hir_def::types::Type as HirType;

use crate::ssa::{ir::function::RuntimeType, opt::flatten_cfg::value_merger::ValueMerger};

use super::{
    basic_block::BasicBlockId,
    call_stack::CallStackId,
    dfg::DataFlowGraph,
    function::Function,
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
///   special support for. (LowLevel)
/// - Opcodes which have no function definition in the
///   source code and must be processed by the IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Intrinsic {
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
            Intrinsic::BlackBox(func) => matches!(
                func,
                BlackBoxFunc::RecursiveAggregation
                    | BlackBoxFunc::MultiScalarMul
                    | BlackBoxFunc::EmbeddedCurveAdd
            ),
        }
    }

    /// Intrinsics which only have a side effect due to the chance that
    /// they can fail a constraint can be deduplicated.
    pub(crate) fn can_be_deduplicated(&self, deduplicate_with_predicate: bool) -> bool {
        match self {
            // These apply a constraint in the form of ACIR opcodes, but they can be deduplicated
            // if the inputs are the same. If they depend on a side effect variable (e.g. because
            // they were in an if-then-else) then `handle_instruction_side_effects` in `flatten_cfg`
            // will have attached the condition variable to their inputs directly, so they don't
            // directly depend on the corresponding `enable_side_effect` instruction any more.
            // However, to conform with the expectations of `Instruction::can_be_deduplicated` and
            // `constant_folding` we only use this information if the caller shows interest in it.
            Intrinsic::ToBits(_)
            | Intrinsic::ToRadix(_)
            | Intrinsic::BlackBox(
                BlackBoxFunc::MultiScalarMul
                | BlackBoxFunc::EmbeddedCurveAdd
                | BlackBoxFunc::RecursiveAggregation,
            ) => deduplicate_with_predicate,

            // Operations that remove items from a slice don't modify the slice, they just assert it's non-empty.
            Intrinsic::SlicePopBack | Intrinsic::SlicePopFront | Intrinsic::SliceRemove => {
                deduplicate_with_predicate
            }

            Intrinsic::AssertConstant
            | Intrinsic::StaticAssert
            | Intrinsic::ApplyRangeConstraint
            | Intrinsic::AsWitness => deduplicate_with_predicate,

            _ => !self.has_side_effects(),
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
pub(crate) enum Endian {
    Big,
    Little,
}

/// Compiler hints.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Hint {
    /// Hint to the compiler to treat the call as having potential side effects,
    /// so that the value passed to it can survive SSA passes without being
    /// simplified out completely. This facilitates testing and reproducing
    /// runtime behavior with constants.
    BlackBox,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
/// Instructions are used to perform tasks.
/// The instructions that the IR is able to specify are listed below.
pub(crate) enum Instruction {
    /// Binary Operations like +, -, *, /, ==, !=
    Binary(Binary),

    /// Converts `Value` into the given NumericType
    Cast(ValueId, NumericType),

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
            | Instruction::Store { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::RangeCheck { .. }
            | Instruction::EnableSideEffectsIf { .. } => InstructionResultType::None,
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

    /// Indicates if the instruction has a side effect, ie. it can fail, or it interacts with memory.
    ///
    /// This is similar to `can_be_deduplicated`, but it doesn't depend on whether the caller takes
    /// constraints into account, because it might not use it to isolate the side effects across branches.
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
                _ => true, // Be conservative and assume other functions can have side effects.
            },

            // These can fail.
            Constrain(..) | RangeCheck { .. } => true,

            // This should never be side-effectful
            MakeArray { .. } => false,

            // Some binary math can overflow or underflow
            Binary(binary) => match binary.operator {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    true
                }
                BinaryOp::Eq
                | BinaryOp::Lt
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::Xor
                | BinaryOp::Shl
                | BinaryOp::Shr => false,
            },

            // These can have different behavior depending on the EnableSideEffectsIf context.
            Cast(_, _)
            | Not(_)
            | Truncate { .. }
            | IfElse { .. }
            | ArrayGet { .. }
            | ArraySet { .. } => self.requires_acir_gen_predicate(dfg),
        }
    }

    /// Indicates if the instruction can be safely replaced with the results of another instruction with the same inputs.
    /// If `deduplicate_with_predicate` is set, we assume we're deduplicating with the instruction
    /// and its predicate, rather than just the instruction. Setting this means instructions that
    /// rely on predicates can be deduplicated as well.
    ///
    /// Some instructions get the predicate attached to their inputs by `handle_instruction_side_effects` in `flatten_cfg`.
    /// These can be deduplicated because they implicitly depend on the predicate, not only when the caller uses the
    /// predicate variable as a key to cache results. However, to avoid tight coupling between passes, we make the deduplication
    /// conditional on whether the caller wants the predicate to be taken into account or not.
    pub(crate) fn can_be_deduplicated(
        &self,
        function: &Function,
        deduplicate_with_predicate: bool,
    ) -> bool {
        use Instruction::*;

        match self {
            // These either have side-effects or interact with memory
            EnableSideEffectsIf { .. }
            | Allocate
            | Load { .. }
            | Store { .. }
            | IncrementRc { .. }
            | DecrementRc { .. } => false,

            Call { func, .. } => match function.dfg[*func] {
                Value::Intrinsic(intrinsic) => {
                    intrinsic.can_be_deduplicated(deduplicate_with_predicate)
                }
                _ => false,
            },

            // We can deduplicate these instructions if we know the predicate is also the same.
            Constrain(..) | RangeCheck { .. } => deduplicate_with_predicate,

            // Arrays can be mutated in unconstrained code so code that handles this case must
            // take care to track whether the array was possibly mutated or not before
            // deduplicating. Since we don't know if the containing pass checks for this, we
            // can only assume these are safe to deduplicate in constrained code.
            MakeArray { .. } => function.runtime().is_acir(),

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
            | ArraySet { .. } => {
                deduplicate_with_predicate || !self.requires_acir_gen_predicate(&function.dfg)
            }
        }
    }

    pub(crate) fn can_eliminate_if_unused(&self, function: &Function) -> bool {
        use Instruction::*;
        match self {
            Binary(binary) => {
                if matches!(binary.operator, BinaryOp::Div | BinaryOp::Mod) {
                    if let Some(rhs) = function.dfg.get_numeric_constant(binary.rhs) {
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
            | ArraySet { .. }
            | MakeArray { .. } => true,

            // Store instructions must be removed by DIE in acir code, any load
            // instructions should already be unused by that point.
            //
            // Note that this check assumes that it is being performed after the flattening
            // pass and after the last mem2reg pass. This is currently the case for the DIE
            // pass where this check is done, but does mean that we cannot perform mem2reg
            // after the DIE pass.
            Store { .. } => {
                matches!(function.runtime(), RuntimeType::Acir(_))
                    && function.reachable_blocks().len() == 1
            }

            Constrain(..)
            | EnableSideEffectsIf { .. }
            | IncrementRc { .. }
            | DecrementRc { .. }
            | RangeCheck { .. } => false,

            // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
            Call { func, .. } => match function.dfg[*func] {
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

    /// If true the instruction will depend on `enable_side_effects` context during acir-gen.
    pub(crate) fn requires_acir_gen_predicate(&self, dfg: &DataFlowGraph) -> bool {
        match self {
            Instruction::Binary(binary) => {
                match binary.operator {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => {
                        // Some binary math can overflow or underflow, but this is only the case
                        // for unsigned types (here we assume the type of binary.lhs is the same)
                        dfg.type_of_value(binary.rhs).is_unsigned()
                    }
                    BinaryOp::Eq
                    | BinaryOp::Lt
                    | BinaryOp::And
                    | BinaryOp::Or
                    | BinaryOp::Xor
                    | BinaryOp::Shl
                    | BinaryOp::Shr => false,
                }
            }

            Instruction::ArrayGet { array, index } => {
                // `ArrayGet`s which read from "known good" indices from an array should not need a predicate.
                !dfg.is_safe_index(*index, *array)
            }

            Instruction::EnableSideEffectsIf { .. } | Instruction::ArraySet { .. } => true,

            Instruction::Call { func, .. } => match dfg[*func] {
                Value::Function(_) => true,
                Value::Intrinsic(intrinsic) => {
                    matches!(intrinsic, Intrinsic::SliceInsert | Intrinsic::SliceRemove)
                }
                _ => false,
            },
            Instruction::Cast(_, _)
            | Instruction::Not(_)
            | Instruction::Truncate { .. }
            | Instruction::Constrain(_, _, _)
            | Instruction::RangeCheck { .. }
            | Instruction::Allocate
            | Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::IfElse { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::MakeArray { .. } => false,
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
            Instruction::MakeArray { elements, typ } => Instruction::MakeArray {
                elements: elements.iter().copied().map(f).collect(),
                typ: typ.clone(),
            },
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
            Instruction::Constrain(lhs, rhs, assert_message) => {
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
            Instruction::ArrayGet { array, index } => {
                *array = f(*array);
                *index = f(*index);
            }
            Instruction::ArraySet { array, index, value, mutable: _ } => {
                *array = f(*array);
                *index = f(*index);
                *value = f(*value);
            }
            Instruction::IncrementRc { value } => *value = f(*value),
            Instruction::DecrementRc { value } => *value = f(*value),
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
        call_stack: CallStackId,
    ) -> SimplifyResult {
        use SimplifyResult::*;
        match self {
            Instruction::Binary(binary) => binary.simplify(dfg),
            Instruction::Cast(value, typ) => simplify_cast(*value, *typ, dfg),
            Instruction::Not(value) => {
                match &dfg[dfg.resolve(*value)] {
                    // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                    // there is no Not on FieldElement, so we'd need to convert between u128. This
                    // would be incorrect however since the extra bits on the field would not be flipped.
                    Value::NumericConstant { constant, typ } if typ.is_unsigned() => {
                        // As we're casting to a `u128`, we need to clear out any upper bits that the NOT fills.
                        let value = !constant.to_u128() % (1 << typ.bit_size());
                        SimplifiedTo(dfg.make_constant(value.into(), *typ))
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
                if let Some(index) = dfg.get_numeric_constant(*index) {
                    try_optimize_array_get_from_previous_set(dfg, *array, index)
                } else {
                    None
                }
            }
            Instruction::ArraySet { array: array_id, index: index_id, value, .. } => {
                let array = dfg.get_array_constant(*array_id);
                let index = dfg.get_numeric_constant(*index_id);
                if let (Some((array, _element_type)), Some(index)) = (array, index) {
                    let index =
                        index.try_to_u32().expect("Expected array index to fit in u32") as usize;

                    if index < array.len() {
                        let elements = array.update(index, *value);
                        let typ = dfg.type_of_value(*array_id);
                        let instruction = Instruction::MakeArray { elements, typ };
                        let new_array = dfg.insert_instruction_and_results(
                            instruction,
                            block,
                            Option::None,
                            call_stack,
                        );
                        return SimplifiedTo(new_array.first());
                    }
                }

                try_optimize_array_set_from_previous_get(dfg, *array_id, *index_id, *value)
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
            Instruction::EnableSideEffectsIf { condition } => {
                if let Some(last) = dfg[block].instructions().last().copied() {
                    let last = &mut dfg[last];
                    if matches!(last, Instruction::EnableSideEffectsIf { .. }) {
                        *last = Instruction::EnableSideEffectsIf { condition: *condition };
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
                let then_condition = dfg.resolve(*then_condition);
                let else_condition = dfg.resolve(*else_condition);
                let typ = dfg.type_of_value(*then_value);

                if let Some(constant) = dfg.get_numeric_constant(then_condition) {
                    if constant.is_one() {
                        return SimplifiedTo(*then_value);
                    } else if constant.is_zero() {
                        return SimplifiedTo(*else_value);
                    }
                }

                let then_value = dfg.resolve(*then_value);
                let else_value = dfg.resolve(*else_value);
                if then_value == else_value {
                    return SimplifiedTo(then_value);
                }

                if let Value::Instruction { instruction, .. } = &dfg[then_value] {
                    if let Instruction::IfElse {
                        then_condition: inner_then_condition,
                        then_value: inner_then_value,
                        else_condition: inner_else_condition,
                        ..
                    } = dfg[*instruction]
                    {
                        if then_condition == inner_then_condition {
                            let instruction = Instruction::IfElse {
                                then_condition,
                                then_value: inner_then_value,
                                else_condition: inner_else_condition,
                                else_value,
                            };
                            return SimplifiedToInstruction(instruction);
                        }
                        // TODO: We could check to see if `then_condition == inner_else_condition`
                        // but we run into issues with duplicate NOT instructions having distinct ValueIds.
                    }
                };

                if let Value::Instruction { instruction, .. } = &dfg[else_value] {
                    if let Instruction::IfElse {
                        then_condition: inner_then_condition,
                        else_condition: inner_else_condition,
                        else_value: inner_else_value,
                        ..
                    } = dfg[*instruction]
                    {
                        if then_condition == inner_then_condition {
                            let instruction = Instruction::IfElse {
                                then_condition,
                                then_value,
                                else_condition: inner_else_condition,
                                else_value: inner_else_value,
                            };
                            return SimplifiedToInstruction(instruction);
                        }
                        // TODO: We could check to see if `then_condition == inner_else_condition`
                        // but we run into issues with duplicate NOT instructions having distinct ValueIds.
                    }
                };

                if matches!(&typ, Type::Numeric(_)) {
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
            Instruction::MakeArray { .. } => None,
        }
    }
}

/// Given a chain of operations like:
/// v1 = array_set [10, 11, 12], index 1, value: 5
/// v2 = array_set v1, index 2, value: 6
/// v3 = array_set v2, index 2, value: 7
/// v4 = array_get v3, index 1
///
/// We want to optimize `v4` to `10`. To do this we need to follow the array value
/// through several array sets. For each array set:
/// - If the index is non-constant we fail the optimization since any index may be changed
/// - If the index is constant and is our target index, we conservatively fail the optimization
///   in case the array_set is disabled from a previous `enable_side_effects_if` and the array get
///   was not.
/// - Otherwise, we check the array value of the array set.
///   - If the array value is constant, we use that array.
///   - If the array value is from a previous array-set, we recur.
fn try_optimize_array_get_from_previous_set(
    dfg: &DataFlowGraph,
    mut array_id: Id<Value>,
    target_index: FieldElement,
) -> SimplifyResult {
    let mut elements = None;

    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        match &dfg[array_id] {
            Value::Instruction { instruction, .. } => {
                match &dfg[*instruction] {
                    Instruction::ArraySet { array, index, value, .. } => {
                        if let Some(constant) = dfg.get_numeric_constant(*index) {
                            if constant == target_index {
                                return SimplifyResult::SimplifiedTo(*value);
                            }

                            array_id = *array; // recur
                        } else {
                            return SimplifyResult::None;
                        }
                    }
                    Instruction::MakeArray { elements: array, typ: _ } => {
                        elements = Some(array.clone());
                        break;
                    }
                    _ => return SimplifyResult::None,
                }
            }
            _ => return SimplifyResult::None,
        }
    }

    if let (Some(array), Some(index)) = (elements, target_index.try_to_u64()) {
        let index = index as usize;
        if index < array.len() {
            return SimplifyResult::SimplifiedTo(array[index]);
        }
    }
    SimplifyResult::None
}

/// If we have an array set whose value is from an array get on the same array at the same index,
/// we can simplify that array set to the array we were looking to perform an array set upon.
///
/// Simple case:
/// v3 = array_get v1, index v2
/// v5 = array_set v1, index v2, value v3
///
/// If we could not immediately simplify the array set from its value, we can try to follow
/// the array set backwards in the case we have constant indices:
///
/// v3 = array_get v1, index 1
/// v5 = array_set v1, index 2, value [Field 100, Field 101, Field 102]
/// v7 = array_set mut v5, index 1, value v3
///
/// We want to optimize `v7` to `v5`. We see that `v3` comes from an array get to `v1`. We follow `v5` backwards and see an array set
/// to `v1` and see that the previous array set occurs to a different constant index.
///
/// For each array_set:
/// - If the index is non-constant we fail the optimization since any index may be changed.
/// - If the index is constant and is our target index, we conservatively fail the optimization.
/// - Otherwise, we check the array value of the `array_set`. We will refer to this array as array'.
///   In the case above, array' is `v1` from `v5 = array set ...`
///   - If the original `array_set` value comes from an `array_get`, check the array in that `array_get` against array'.
///   - If the two values are equal we can simplify.
///     - Continuing the example above, as we have `v3 = array_get v1, index 1`, `v1` is
///       what we want to check against array'. We now know we can simplify `v7` to `v5` as it is unchanged.
///   - If they are not equal, recur marking the current `array_set` array as the new array id to use in the checks
fn try_optimize_array_set_from_previous_get(
    dfg: &DataFlowGraph,
    mut array_id: ValueId,
    target_index: ValueId,
    target_value: ValueId,
) -> SimplifyResult {
    let array_from_get = match &dfg[target_value] {
        Value::Instruction { instruction, .. } => match &dfg[*instruction] {
            Instruction::ArrayGet { array, index } => {
                if *array == array_id && *index == target_index {
                    // If array and index match from the value, we can immediately simplify
                    return SimplifyResult::SimplifiedTo(array_id);
                } else if *index == target_index {
                    *array
                } else {
                    return SimplifyResult::None;
                }
            }
            _ => return SimplifyResult::None,
        },
        _ => return SimplifyResult::None,
    };

    // At this point we have determined that the value we are writing in the `array_set` instruction
    // comes from an `array_get` from the same index at which we want to write it at.
    // It's possible that we're acting on the same array where other indices have been mutated in between
    // the `array_get` and `array_set` (resulting in the `array_id` not matching).
    //
    // We then inspect the set of `array_set`s which which led to the current array the `array_set` is acting on.
    // If we can work back to the array on which the `array_get` was reading from without having another `array_set`
    // act on the same index then we can be sure that the new `array_set` can be removed without affecting the final result.
    let Some(target_index) = dfg.get_numeric_constant(target_index) else {
        return SimplifyResult::None;
    };

    let original_array_id = array_id;
    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        match &dfg[array_id] {
            Value::Instruction { instruction, .. } => match &dfg[*instruction] {
                Instruction::ArraySet { array, index, .. } => {
                    let Some(index) = dfg.get_numeric_constant(*index) else {
                        return SimplifyResult::None;
                    };

                    if index == target_index {
                        return SimplifyResult::None;
                    }

                    if *array == array_from_get {
                        return SimplifyResult::SimplifiedTo(original_array_id);
                    }

                    array_id = *array; // recur
                }
                _ => return SimplifyResult::None,
            },
            _ => return SimplifyResult::None,
        }
    }

    SimplifyResult::None
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
pub(crate) enum ConstrainError {
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
}

impl TerminatorInstruction {
    /// Map each ValueId in this terminator to a new value.
    pub(crate) fn map_values(
        &self,
        mut f: impl FnMut(ValueId) -> ValueId,
    ) -> TerminatorInstruction {
        use TerminatorInstruction::*;
        match self {
            JmpIf { condition, then_destination, else_destination, call_stack } => JmpIf {
                condition: f(*condition),
                then_destination: *then_destination,
                else_destination: *else_destination,
                call_stack: *call_stack,
            },
            Jmp { destination, arguments, call_stack } => Jmp {
                destination: *destination,
                arguments: vecmap(arguments, |value| f(*value)),
                call_stack: *call_stack,
            },
            Return { return_values, call_stack } => Return {
                return_values: vecmap(return_values, |value| f(*value)),
                call_stack: *call_stack,
            },
        }
    }

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

    pub(crate) fn call_stack(&self) -> CallStackId {
        match self {
            TerminatorInstruction::JmpIf { call_stack, .. }
            | TerminatorInstruction::Jmp { call_stack, .. }
            | TerminatorInstruction::Return { call_stack, .. } => *call_stack,
        }
    }

    pub(crate) fn set_call_stack(&mut self, new_call_stack: CallStackId) {
        match self {
            TerminatorInstruction::JmpIf { call_stack, .. }
            | TerminatorInstruction::Jmp { call_stack, .. }
            | TerminatorInstruction::Return { call_stack, .. } => *call_stack = new_call_stack,
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
