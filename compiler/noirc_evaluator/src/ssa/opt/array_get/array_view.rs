//! The [`ArrayView`] cache used by [`array_get_optimization`][super::Function::array_get_optimization].
//!
//! An [`ArrayView`] records the known contents of an array value: the values known to live at
//! specific constant indices (each with the side-effects predicate it was written under) over a
//! [base][ArrayBase] that says where any other index gets its value from. The pass builds one view
//! per array value it scans and resolves a constant-index `array_get` with [`ArrayView::resolve`],
//! a lookup rather than a walk back over previous instructions.
//!
//! The fields are private to this module: outside code seeds a view with [`ArrayView::for_value`],
//! extends it with [`ArrayView::with_element`], and reads it with [`ArrayView::resolve`], so the
//! predicate rule that makes a cached element safe to use lives in one place.
use im::OrdMap;

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::Instruction,
    types::Type,
    value::{Value, ValueId},
};

/// The known contents of an array value, built up incrementally by
/// [`array_get_optimization`][super::Function::array_get_optimization] as it scans a function.
#[derive(Clone)]
pub(super) struct ArrayView {
    /// Values known to live at specific constant indices, overriding `base`.
    elements: OrdMap<u32, KnownElement>,
    /// Where an index that isn't present in `elements` gets its value from.
    base: ArrayBase,
}

#[derive(Clone, Copy)]
struct KnownElement {
    value: ValueId,
    /// The side-effects predicate under which this element was written by an `array_set`.
    predicate: ValueId,
}

#[derive(Clone)]
enum ArrayBase {
    /// Indices not in `elements` come from this `make_array`'s elements, which are stored in logical
    /// order, so a constant index reads that element directly.
    MakeArray { elements: im::Vector<ValueId> },
    /// Indices not in `elements` can be read directly from this array (a function parameter), at
    /// the same index. `length` bounds which indices that is valid for.
    ReadFrom { array: ValueId, length: u32 },
    /// Nothing is known about indices not in `elements`.
    Unknown,
}

/// How an `array_get` at a known index can be resolved against an [`ArrayView`].
pub(super) enum Resolution {
    /// The `array_get` is equal to this value.
    Value(ValueId),
    /// The `array_get` can read from this array instead, at the same index.
    ReadFrom(ValueId),
}

impl ArrayView {
    /// Seeds the view of an array that hasn't been written by an `array_set` earlier in the current
    /// block: a `make_array` (local or global) exposes its elements directly, a parameter can be
    /// read from at the same index, and anything else (including arrays from other blocks) is
    /// opaque.
    pub(super) fn for_value(dfg: &DataFlowGraph, array: ValueId) -> Self {
        if let Some((Instruction::MakeArray { elements, .. }, _)) =
            dfg.get_local_or_global_instruction_with_id(array)
        {
            return ArrayView::from_make_array(elements.clone());
        }

        if let Value::Param { typ: Type::Array(_, length), .. } = &dfg[array] {
            return ArrayView {
                elements: OrdMap::new(),
                base: ArrayBase::ReadFrom { array, length: length.0 },
            };
        }

        ArrayView::unknown()
    }

    /// Records that `value` lives at `index`, written under `predicate`, overriding whatever the
    /// base or an earlier write said about that index.
    pub(super) fn with_element(mut self, index: u32, value: ValueId, predicate: ValueId) -> Self {
        self.elements.insert(index, KnownElement { value, predicate });
        self
    }

    /// Resolves an `array_get` of `array` at a known `index` under `predicate`, if the view knows
    /// what it reads. `array` is the value being read, passed so a read is never rewritten to fetch
    /// from itself.
    pub(super) fn resolve(
        &self,
        array: ValueId,
        index: u32,
        predicate: ValueId,
        dfg: &DataFlowGraph,
    ) -> Option<Resolution> {
        if let Some(element) = self.elements.get(&index) {
            // A known element can only be used if it was written unconditionally or under the same
            // predicate as the `array_get`; otherwise the write might not have happened.
            //
            // Once the index is known to have been written under *any* predicate the write may have
            // happened, so we must not fall through to `base`: `base` is fixed to the original array
            // when the view is seeded and never updated as writes accumulate, so it no longer
            // describes this index. Return here even on a predicate mismatch, leaving the
            // `array_get` in place, rather than reading a stale value from `base`.
            return (dfg.is_constant_true(element.predicate) || element.predicate == predicate)
                .then_some(Resolution::Value(element.value));
        }

        match self.base {
            // `make_array` elements are in logical order, so a constant index reads directly.
            ArrayBase::MakeArray { ref elements } => {
                elements.get(index as usize).copied().map(Resolution::Value)
            }
            // Reading directly from `array` itself wouldn't be an improvement.
            ArrayBase::ReadFrom { array: source, length } => {
                (index < length && source != array).then_some(Resolution::ReadFrom(source))
            }
            ArrayBase::Unknown => None,
        }
    }

    fn from_make_array(elements: im::Vector<ValueId>) -> Self {
        ArrayView { elements: OrdMap::new(), base: ArrayBase::MakeArray { elements } }
    }

    fn unknown() -> Self {
        ArrayView { elements: OrdMap::new(), base: ArrayBase::Unknown }
    }
}
