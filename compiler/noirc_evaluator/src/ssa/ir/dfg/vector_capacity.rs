//! Static capacity of vectors.
//!
//! A vector value has a backing array, whose length is its capacity, and a separate semantic
//! length that is passed alongside it and can be smaller. Passes that build or replace vectors
//! need a capacity that is known at compile time:
//!
//! - [`DataFlowGraph::try_get_vector_capacity`] and
//!   [`DataFlowGraph::try_get_vector_backing_capacity`] trace a vector back to the `make_array`
//!   it was built from.
//! - Remove IfElse tracks capacities instruction by instruction while it merges vectors.
//! - The SSA interpreter sizes the results of vector intrinsics called while side effects are
//!   disabled.
//!
//! All of them use [`vector_capacity_flows`] to find how an intrinsic's vector result relates to
//! its vector argument, and [`constant_vector_lengths`] to find the vector arguments whose
//! semantic length is a known constant.
use acvm::{
    AcirField,
    acir::brillig::lengths::{SemanticLength, SemiFlattenedLength},
};
use rustc_hash::FxHashMap as HashMap;

use crate::{
    brillig::assert_u32,
    ssa::ir::{
        instruction::{Hint, Instruction, Intrinsic},
        types::Type,
        value::{Value, ValueId},
    },
};

use super::DataFlowGraph;

/// How the capacity of a vector intrinsic's result relates to the capacity of its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapacityChange {
    /// The result has the same capacity as the input.
    Same,
    /// The result holds one more element than the input. A push or insert always grows the
    /// backing array, even when the semantic length was below the capacity.
    Grow,
    /// The result holds one fewer element than the input.
    Shrink,
}

impl CapacityChange {
    /// The capacity of the result, or `None` if it does not fit in a `u32`.
    pub(crate) fn apply(self, capacity: SemanticLength) -> Option<SemanticLength> {
        match self {
            CapacityChange::Same => Some(capacity),
            CapacityChange::Grow => capacity.0.checked_add(1).map(SemanticLength),
            // Popping from an empty vector fails, so saturating keeps the capacity of the
            // (unused) result well defined.
            CapacityChange::Shrink => Some(SemanticLength(capacity.0.saturating_sub(1))),
        }
    }
}

/// A vector (or, for `as_vector`, array) argument of an intrinsic call and the vector result
/// whose capacity follows from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapacityFlow {
    pub(crate) input: ValueId,
    pub(crate) output: ValueId,
    pub(crate) change: CapacityChange,
}

/// The vector results of a call to `intrinsic` whose capacity follows from one of its arguments.
pub(crate) fn vector_capacity_flows(
    dfg: &DataFlowGraph,
    intrinsic: Intrinsic,
    arguments: &[ValueId],
    results: &[ValueId],
) -> Vec<CapacityFlow> {
    let flow = |output: ValueId, change| {
        let input = arguments[1];
        debug_assert!(matches!(*dfg.type_of_value(input), Type::Vector(_)));
        debug_assert!(matches!(*dfg.type_of_value(output), Type::Vector(_)));
        vec![CapacityFlow { input, output, change }]
    };
    match intrinsic {
        // (len, vector, ...) -> (len, vector)
        Intrinsic::VectorPushBack | Intrinsic::VectorPushFront | Intrinsic::VectorInsert => {
            flow(results[1], CapacityChange::Grow)
        }
        // (len, vector, ...) -> (len, vector, ...item)
        Intrinsic::VectorPopBack | Intrinsic::VectorRemove => {
            flow(results[1], CapacityChange::Shrink)
        }
        // (len, vector) -> (...item, len, vector)
        Intrinsic::VectorPopFront => flow(results[results.len() - 1], CapacityChange::Shrink),
        // (array) -> (len, vector)
        Intrinsic::AsVector => {
            vec![CapacityFlow {
                input: arguments[0],
                output: results[1],
                change: CapacityChange::Same,
            }]
        }
        // The hint returns its arguments unchanged.
        Intrinsic::Hint(Hint::BlackBox) => arguments
            .iter()
            .zip(results)
            .filter(|(argument, _)| matches!(*dfg.type_of_value(**argument), Type::Vector(_)))
            .map(|(input, output)| CapacityFlow {
                input: *input,
                output: *output,
                change: CapacityChange::Same,
            })
            .collect(),
        Intrinsic::AssertConstant
        | Intrinsic::StaticAssert
        | Intrinsic::ApplyRangeConstraint
        | Intrinsic::ArrayLen
        | Intrinsic::ArrayAsStrUnchecked
        | Intrinsic::StrAsBytes
        | Intrinsic::BlackBox(_)
        | Intrinsic::AsWitness
        | Intrinsic::IsUnconstrained
        | Intrinsic::DerivePedersenGenerators
        | Intrinsic::ToBits(_)
        | Intrinsic::ToRadix(_)
        | Intrinsic::ArrayRefCount
        | Intrinsic::VectorRefCount
        | Intrinsic::FieldLessThan => Vec::new(),
    }
}

/// The vector arguments of a call to `intrinsic` whose semantic length is passed as a constant
/// in the argument before them.
pub(crate) fn constant_vector_lengths(
    dfg: &DataFlowGraph,
    intrinsic: Intrinsic,
    arguments: &[ValueId],
) -> Vec<(ValueId, SemanticLength)> {
    let constant_length = |length: ValueId| {
        dfg.get_numeric_constant(length)
            .map(|length| SemanticLength(length.try_to_u32().expect("Type should be u32")))
    };
    match intrinsic {
        Intrinsic::VectorPushBack
        | Intrinsic::VectorPushFront
        | Intrinsic::VectorInsert
        | Intrinsic::VectorPopBack
        | Intrinsic::VectorRemove
        | Intrinsic::VectorPopFront => {
            constant_length(arguments[0]).map(|length| (arguments[1], length)).into_iter().collect()
        }
        Intrinsic::Hint(Hint::BlackBox) => arguments
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, argument)| matches!(*dfg.type_of_value(**argument), Type::Vector(_)))
            .filter_map(|(i, argument)| {
                debug_assert!(matches!(*dfg.type_of_value(arguments[i - 1]), Type::Numeric(_)));
                constant_length(arguments[i - 1]).map(|length| (*argument, length))
            })
            .collect(),
        _ => Vec::new(),
    }
}

impl DataFlowGraph {
    /// Try to find out the capacity of a vector by tracing it back to a `MakeArray`.
    ///
    /// The result of a vector intrinsic whose length argument is a known constant is sized from
    /// that length, which can be smaller than its backing array. Use
    /// [`Self::try_get_vector_backing_capacity`] where the full backing array is needed.
    pub(crate) fn try_get_vector_capacity(&self, value: ValueId) -> Option<SemanticLength> {
        CapacityTracer { dfg: self, use_constant_length: true, cache: HashMap::default() }
            .capacity(value)
    }

    /// Try to find out the size of the backing array of a vector by tracing it back to a
    /// `MakeArray`, ignoring the semantic length of any vector intrinsic along the way.
    ///
    /// Earlier passes may have emitted reads of every element of the backing array, so a value
    /// that stands in for the vector must be at least this large.
    pub(crate) fn try_get_vector_backing_capacity(&self, value: ValueId) -> Option<SemanticLength> {
        CapacityTracer { dfg: self, use_constant_length: false, cache: HashMap::default() }
            .capacity(value)
    }
}

struct CapacityTracer<'dfg> {
    dfg: &'dfg DataFlowGraph,
    use_constant_length: bool,
    /// Capacities computed so far: the values merged by `IfElse` instructions can share
    /// inputs, and without this the walk is exponential in the depth of nested merges.
    cache: HashMap<ValueId, Option<SemanticLength>>,
}

impl CapacityTracer<'_> {
    fn capacity(&mut self, value: ValueId) -> Option<SemanticLength> {
        if let Some(capacity) = self.cache.get(&value) {
            return *capacity;
        }
        let capacity = self.compute(value);
        self.cache.insert(value, capacity);
        capacity
    }

    fn compute(&mut self, value: ValueId) -> Option<SemanticLength> {
        let dfg = self.dfg;
        // For arrays we know the size statically
        if let Some(length) = dfg.try_get_array_length(value) {
            return Some(length);
        }

        let (instruction, instruction_id) = dfg.get_local_or_global_instruction_with_id(value)?;
        match instruction {
            Instruction::MakeArray { .. } => {
                let (array, typ) = dfg.get_array_constant(value)?;
                Some(make_array_capacity(array.len(), &typ))
            }
            Instruction::ArraySet { array, .. } | Instruction::ArrayGet { array, .. } => {
                self.capacity(*array)
            }
            Instruction::Call { func, arguments } => {
                let Value::Intrinsic(intrinsic) = dfg[*func] else {
                    return None;
                };
                let results = dfg.instruction_results(instruction_id);
                let flow = vector_capacity_flows(dfg, intrinsic, arguments, results)
                    .into_iter()
                    .find(|flow| flow.output == value)?;
                // It should be okay to use a constant semantic length; for example the
                // ValueMerger would get fewer items.
                let known_length = self
                    .use_constant_length
                    .then(|| {
                        constant_vector_lengths(dfg, intrinsic, arguments)
                            .into_iter()
                            .find_map(|(vector, length)| (vector == flow.input).then_some(length))
                    })
                    .flatten();
                let input_capacity = match known_length {
                    Some(length) => length,
                    None => self.capacity(flow.input)?,
                };
                flow.change.apply(input_capacity)
            }
            Instruction::IfElse { then_value, else_value, .. } => {
                // The capacity is the longer of the two after merging.
                let then_capacity = self.capacity(*then_value)?;
                let else_capacity = self.capacity(*else_value)?;
                Some(SemanticLength(std::cmp::max(then_capacity.0, else_capacity.0)))
            }
            _ => None,
        }
    }
}

/// The capacity of a vector built by a `make_array` of `elements` flattened values.
fn make_array_capacity(elements: usize, typ: &Type) -> SemanticLength {
    let elements_size = typ.element_size();
    if elements_size.0 == 0 {
        SemanticLength(assert_u32(elements))
    } else {
        SemiFlattenedLength(assert_u32(elements)) / elements_size
    }
}
