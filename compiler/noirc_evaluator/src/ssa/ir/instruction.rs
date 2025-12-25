use noirc_errors::call_stack::CallStackId;
use rustc_stable_hash::{FromStableHash, SipHasher128Hash};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use acvm::{
    AcirField,
    acir::{BlackBoxFunc, circuit::ErrorSelector},
};
use iter_extended::vecmap;
use noirc_frontend::hir_def::types::Type as HirType;

use crate::ssa::{ir::integer::IntegerConstant, opt::pure::Purity};

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
    /// ArrayLen - returns the length of the input array
    /// argument: array (value id)
    /// result: length of the array, panic if the input is not an array
    ArrayLen,
    /// ArrayAsStrUnchecked - Converts a byte array of type `[u8; N]` to a string
    /// argument: array (value id)
    /// result: str
    ArrayAsStrUnchecked,
    /// AsVector
    /// argument: value id
    /// result: a vector containing the elements of the argument. Panic if the value id does not correspond to an `array` type
    AsVector,
    /// AssertConstant - Enforce the argument to be a constant value, at compile time.
    /// argument: value id
    /// result: (), panic if the argument does not resolve to a constant value
    AssertConstant,
    /// StaticAssert - Enforce the first argument to be true, at compile time
    /// arguments: boolean (value id), ...message. The message can be a `format string` of several arguments
    /// result: (), panic if the arguments do not resolve to constant values or if the first one is false.
    StaticAssert,
    /// VectorPushBack - Add elements at the end of a vector
    /// arguments:  vector length, vector contents, ...elements_to_push
    /// result: a vector containing `vector contents,..elements_to_push`
    VectorPushBack,
    /// VectorPushFront - Add elements at the start of a vector
    /// arguments:  vector length, vector contents, ...elements_to_push
    /// result: a vector containing `..elements_to_push, vector contents`
    VectorPushFront,
    /// VectorPopBack - Removes the last element of a vector
    /// arguments: vector length, vector contents
    /// result: a vector without the last element of `vector contents`
    VectorPopBack,
    /// VectorPopFront - Removes the first element of a vector
    /// arguments: vector length, vector contents
    /// result: a vector without the first element of `vector contents`
    VectorPopFront,
    /// VectorInsert - Insert elements inside a vector.
    /// arguments: vector length, vector contents, insert index, ...elements_to_insert
    /// result: a vector with ...elements_to_insert inserted at the `insert index`
    VectorInsert,
    /// VectorRemove - Removes an element from a vector
    /// arguments: vector length, vector contents, remove index
    /// result: a vector with without the element at `remove index`
    VectorRemove,
    /// ApplyRangeConstraint - Enforces the `bit size` of the first argument via a range check.
    /// arguments: value id, bit size (constant)
    /// result: applies a range check constraint to the input. It is replaced by a RangeCheck instruction during simplification.
    ApplyRangeConstraint,
    /// StrAsBytes - Convert a `str` into a byte array of type `[u8; N]`
    /// arguments: value id
    /// result: the argument. Internally a `str` is a byte array.
    StrAsBytes,
    /// ToBits(Endian) - Computes the bit decomposition of the argument.
    /// argument: a field element (value id)
    /// result: an array whose elements are the bit decomposition of the argument, in the endian order depending on the chosen variant.
    /// The type of the result gives the number of limbs to use for the decomposition.
    ToBits(Endian),
    /// ToRadix(Endian) - Decompose the first argument over the `radix` base
    /// arguments: a field element (value id), the radix to use (constant, a power of 2 between 2 and 256)
    /// result: an array whose elements are the decomposition of the argument into the `radix` base, in the endian order depending on the chosen variant.
    /// The type of the result gives the number of limbs to use for the decomposition.
    ToRadix(Endian),
    /// BlackBox(BlackBoxFunc) - Calls a blackbox function. More details can be found here: [acvm-repo::acir::::circuit::opcodes::BlackBoxFuncCall]
    BlackBox(BlackBoxFunc),
    /// Hint(Hint) - Avoid its arguments to be removed by DIE.
    /// arguments: ... value id
    /// result: the arguments. Hint does not layout any constraint but avoid its arguments to be simplified out during SSA transformations
    Hint(Hint),
    /// AsWitness - Adds a new witness constrained to be equal to the argument
    /// arguments: value id
    /// result: the argument
    AsWitness,
    /// IsUnconstrained - Indicates if the execution context is constrained or unconstrained
    /// argument: ()
    /// result: true if execution is under unconstrained context, false else.
    IsUnconstrained,
    /// DerivePedersenGenerators - Computes the Pedersen generators
    /// arguments: domain_separator_string (constant string), starting_index (constant)
    /// result: array of elliptic curve points (Grumpkin) containing the generators.
    /// The type of the result gives the number of generators to compute.
    DerivePedersenGenerators,
    /// FieldLessThan - Compare the arguments: `lhs` < `rhs`
    /// arguments: lhs, rhs. Field elements
    /// result: true if `lhs` mod p < `rhs` mod p (p being the field characteristic), false else
    FieldLessThan,
    /// ArrayRefCount - Gives the reference count of the array
    /// argument: array (value id)
    /// result: reference count of `array`. In unconstrained context, the reference count is stored alongside the array.
    /// in constrained context, it will be 0.
    ArrayRefCount,
    /// VectorRefCount - Gives the reference count of the vector
    /// arguments: vector length, vector contents (value id)
    /// result: reference count of `vector`. In unconstrained context, the reference count is stored alongside the vector.
    /// in constrained context, it will be 0.
    VectorRefCount,
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::ArrayLen => write!(f, "array_len"),
            Intrinsic::ArrayAsStrUnchecked => write!(f, "array_as_str_unchecked"),
            Intrinsic::AsVector => write!(f, "as_vector"),
            Intrinsic::AssertConstant => write!(f, "assert_constant"),
            Intrinsic::StaticAssert => write!(f, "static_assert"),
            Intrinsic::VectorPushBack => write!(f, "vector_push_back"),
            Intrinsic::VectorPushFront => write!(f, "vector_push_front"),
            Intrinsic::VectorPopBack => write!(f, "vector_pop_back"),
            Intrinsic::VectorPopFront => write!(f, "vector_pop_front"),
            Intrinsic::VectorInsert => write!(f, "vector_insert"),
            Intrinsic::VectorRemove => write!(f, "vector_remove"),
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
            Intrinsic::VectorRefCount => write!(f, "vector_refcount"),
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
            // Array & vector ref counts are treated as having side effects since they operate
            // on hidden variables on otherwise identical array values.
            | Intrinsic::ArrayRefCount
            | Intrinsic::VectorRefCount
            | Intrinsic::AsWitness => true,

            // These apply a constraint that the input must fit into a specified number of limbs.
            Intrinsic::ToBits(_) | Intrinsic::ToRadix(_) => true,

            // These imply a check that the vector is non-empty and should fail otherwise.
            Intrinsic::VectorPopBack | Intrinsic::VectorPopFront | Intrinsic::VectorRemove | Intrinsic::VectorInsert => true,

            Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::AsVector
            | Intrinsic::VectorPushBack
            | Intrinsic::VectorPushFront
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

            // Operations that remove items from a vector don't modify the vector, they just assert it's non-empty.
            // Vector insert also reads from its input vector, thus needing to assert that it is non-empty.
            Intrinsic::VectorPopBack
            | Intrinsic::VectorPopFront
            | Intrinsic::VectorRemove
            | Intrinsic::VectorInsert => Purity::PureWithPredicate,

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
            "as_vector" => Some(Intrinsic::AsVector),
            "assert_constant" => Some(Intrinsic::AssertConstant),
            "static_assert" => Some(Intrinsic::StaticAssert),
            "apply_range_constraint" => Some(Intrinsic::ApplyRangeConstraint),
            "vector_push_back" => Some(Intrinsic::VectorPushBack),
            "vector_push_front" => Some(Intrinsic::VectorPushFront),
            "vector_pop_back" => Some(Intrinsic::VectorPopBack),
            "vector_pop_front" => Some(Intrinsic::VectorPopFront),
            "vector_insert" => Some(Intrinsic::VectorInsert),
            "vector_remove" => Some(Intrinsic::VectorRemove),
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
            "vector_refcount" => Some(Intrinsic::VectorRefCount),

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

    /// Creates a new array or vector.
    ///
    /// `typ` should be an array or vector type with an element type
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
            Instruction::Binary(binary) => binary.has_side_effects(dfg),

            Instruction::ArrayGet { array, index } => {
                // `ArrayGet`s which read from "known good" indices from an array should not need a predicate.
                // This extra out of bounds (OOB) check is only inserted in the ACIR runtime.
                // Thus, in Brillig an `ArrayGet` is always a pure operation in isolation and
                // it is expected that OOB checks are inserted separately. However, it would
                // not be safe to separate the `ArrayGet` from the OOB constraints that precede it,
                // because while it could read an array index, the returned data could be invalid,
                // and fail at runtime if we tried using it in the wrong context.
                !dfg.is_safe_index(*index, *array)
            }

            Instruction::EnableSideEffectsIf { .. }
            | Instruction::ArraySet { .. }
            | Instruction::ConstrainNotEqual(..) => true,

            Instruction::Call { func, .. } => match dfg[*func] {
                Value::Function(id) => !matches!(dfg.purity_of(id), Some(Purity::Pure)),
                Value::Intrinsic(intrinsic) => {
                    match intrinsic {
                        // These utilize `noirc_evaluator::acir::Context::get_flattened_index` internally
                        // which uses the side effects predicate.
                        Intrinsic::VectorInsert | Intrinsic::VectorRemove => true,
                        // Technically these don't use the side effects predicate, but they fail on empty vectors,
                        // and by pretending that they require the predicate, we can preserve any current side
                        // effect variable in the SSA and use it to optimize out memory operations that we know
                        // would fail, but they shouldn't because they might be disabled.
                        Intrinsic::VectorPopFront | Intrinsic::VectorPopBack => true,
                        _ => false,
                    }
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

            // Some binary math can overflow or underflow.
            Binary(binary) => binary.has_side_effects(dfg),

            // These don't have side effects
            Cast(_, _) | Not(_) | Truncate { .. } | IfElse { .. } => false,

            // `ArrayGet`s which read from "known good" indices from an array have no side effects
            // This extra out of bounds (OOB) check is only inserted in the ACIR runtime.
            // Thus, in Brillig an `ArrayGet` is always a pure operation in isolation and
            // it is expected that OOB checks are inserted separately. However, it would not
            // be safe to separate the `ArrayGet` from its corresponding OOB constraints in Brillig,
            // as a value read from an array at an invalid index could cause failures when subsequently
            // used in the wrong context. Since we use this information to decide whether to hoist
            // instructions during deduplication, we consider unsafe values as potentially having
            // indirect side effects.
            ArrayGet { array, index } => !dfg.is_safe_index(*index, *array),

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
            Instruction::MakeArray { elements, typ } => {
                let mut elements = elements.clone();
                im_vec_map_values_mut(&mut elements, f);
                Instruction::MakeArray { elements, typ: typ.clone() }
            }
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
                im_vec_map_values_mut(elements, f);
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
    Vector,
}

impl ArrayOffset {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Array),
            3 => Some(Self::Vector),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Self::None => 0,
            // Arrays in brillig are represented as [RC, ...items]
            Self::Array => 1,
            // Vectors in brillig are represented as [RC, Size, Capacity, ...items]
            Self::Vector => 3,
        }
    }
}

impl Binary {
    pub(crate) fn has_side_effects(&self, dfg: &DataFlowGraph) -> bool {
        match self.operator {
            BinaryOp::Add { unchecked: false }
            | BinaryOp::Sub { unchecked: false }
            | BinaryOp::Mul { unchecked: false } => {
                let typ = dfg.type_of_value(self.lhs);
                !matches!(typ, Type::Numeric(NumericType::NativeField))
            }
            BinaryOp::Div | BinaryOp::Mod => {
                // If we don't know rhs at compile time, it might be zero or -1
                let Some(rhs) = dfg.get_numeric_constant(self.rhs) else {
                    return true;
                };

                // Div or mod by zero is a side effect (failure)
                if rhs.is_zero() {
                    return true;
                }

                // For signed types, division or modulo by -1 can overflow.
                let typ = dfg.type_of_value(self.rhs).unwrap_numeric();
                let NumericType::Signed { bit_size } = typ else {
                    return false;
                };

                let minus_one = IntegerConstant::Signed { value: -1, bit_size };
                if IntegerConstant::from_numeric_constant(rhs, typ) == Some(minus_one) {
                    return true;
                }

                false
            }
            BinaryOp::Shl | BinaryOp::Shr => {
                // Bit-shifts which are known to be by a number of bits less than the bit size of the type have no side effects.
                dfg.get_numeric_constant(self.rhs).is_none_or(|c| {
                    let typ = dfg.type_of_value(self.lhs);
                    c >= typ.bit_size().into()
                })
            }
            BinaryOp::Add { unchecked: true }
            | BinaryOp::Sub { unchecked: true }
            | BinaryOp::Mul { unchecked: true }
            | BinaryOp::Eq
            | BinaryOp::Lt
            | BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Xor => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ErrorType {
    String(String),
    Dynamic(HirType),
}

impl ErrorType {
    /// Hash the error type to get a unique selector for it.
    pub fn selector(&self) -> ErrorSelector {
        struct U64(pub u64);

        impl FromStableHash for U64 {
            type Hash = SipHasher128Hash;

            fn from(hash: Self::Hash) -> Self {
                Self(hash.0[0])
            }
        }

        // We explicitly do not use `rustc-hash` here as we require hashes to be stable across 32- and 64-bit architectures.
        let mut hasher =
            rustc_stable_hash::StableHasher::<rustc_stable_hash::hashers::SipHasher128>::new();
        self.hash(&mut hasher);
        let hash = hasher.finish::<U64>();
        ErrorSelector::new(hash.0)
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

/// Try to avoid mutation until we know something changed, to take advantage of
/// structural sharing, and avoid needlessly calling `Arc::make_mut` which clones
/// the content and increases memory use by allocating more pointers on the heap.
fn im_vec_map_values_mut<T, F>(xs: &mut im::Vector<T>, mut f: F)
where
    T: Copy + PartialEq,
    F: FnMut(T) -> T,
{
    // Even `xs.iter_mut()` calls `get_mut` on each element, regardless of whether there is actual mutation.
    // If we go index-by-index, get the item, put it back only if it changed, then we can avoid
    // allocating memory unless we need to, however we incur O(n * log(n)) complexity.
    // Collecting changes first and then updating only those positions proved to be the
    // fastest among some alternatives that didn't sacrifice memory for speed or vice versa.
    let mut changes = Vec::new();
    for (i, x) in xs.iter().enumerate() {
        let y = f(*x);
        if *x != y {
            changes.push((i, y));
        }
    }
    if changes.is_empty() {
        return;
    }
    // Using `Focus` allows us to only make mutable what is needed,
    // and should be faster for batches than indexing individual items.
    let mut focus = xs.focus_mut();
    for (i, y) in changes {
        focus.set(i, y);
    }
}
