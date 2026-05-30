//! Conservative value-range analysis for SSA values.
//!
//! [`Analysis`] computes, for each numeric SSA value, an over-approximation of the set of values it
//! can take at runtime. Downstream passes use these ranges to remove or shrink constraints — eliding
//! redundant range checks, narrowing `Lt` comparison widths, and converting checked arithmetic to
//! unchecked — so the ranges feed directly into circuit size.
//!
//! Because the results drop constraints, every inferred range **must be a sound over-approximation**:
//! it must contain every value the SSA value can actually take. A range that is too *tight* would
//! drop a needed constraint and under-constrain the circuit; a range that is too *loose* only costs
//! optimization. The analysis therefore only ever narrows from a sound starting point (the value's
//! full type range) and rejects empty intersections rather than inventing a bound.
//!
//! Ranges are derived three ways, run to a fixed point over the function's instructions:
//! - **forward** inference of an instruction's result range from its operand ranges;
//! - **backward** inference of an operand range from a known result range;
//! - **constraint** propagation from `RangeCheck` and `Constrain` instructions.
//!
//! Backward and constraint propagation rely on a fact that only holds where a given instruction
//! executes, so they run only when every instruction executes unconditionally (a single block with
//! no predication). Forward inference is always sound and always runs.

use acvm::AcirField;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::{
    instruction::{Binary, BinaryOp, Instruction},
    types::{NumericType, Type, max_unsigned_value_for_bit_size},
    value::{Value, ValueId},
};

use super::DataFlowGraph;

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    debug_assert!(denominator > 0);

    let quotient = numerator / denominator;
    quotient + u128::from(numerator % denominator != 0)
}

/// Computes conservative unsigned value ranges for SSA values.
pub(super) struct Analysis<'dfg> {
    dfg: &'dfg DataFlowGraph,
}

#[derive(Clone, Copy)]
enum RangeSource<'facts> {
    Recursive,
    Facts(&'facts Facts),
}

impl<'facts> RangeSource<'facts> {
    fn range(self, analysis: &Analysis<'_>, value: ValueId) -> Option<Range> {
        match self {
            Self::Recursive => analysis.range(value),
            Self::Facts(facts) => facts.range(value),
        }
    }

    /// Use fallback ranges only during recursive analysis.
    ///
    /// Fixed-point propagation must not invent facts for missing operands.
    fn range_or_fallback(
        self,
        analysis: &Analysis<'_>,
        value: ValueId,
        fallback: Range,
    ) -> Option<Range> {
        match self {
            Self::Recursive => Some(analysis.range(value).unwrap_or(fallback)),
            Self::Facts(facts) => facts.range(value),
        }
    }
}

impl<'dfg> Analysis<'dfg> {
    pub(super) fn new(dfg: &'dfg DataFlowGraph) -> Self {
        Self { dfg }
    }

    /// Returns the maximum number of bits that `value` can occupy.
    ///
    /// A numeric constant yields the exact width it requires. Otherwise this runs the range
    /// analysis — forward inference over the function's instructions, plus backward inference within
    /// a single unpredicated block — and returns the tightest width the inferred bounds allow,
    /// bounded by the value's type width.
    pub(super) fn bits(&self, value: ValueId) -> u32 {
        if let Some(range) = self.range(value) {
            return self.dfg.type_of_value(value).bit_size().min(range.max_bits());
        }

        match self.dfg[value] {
            Value::Instruction { instruction, .. } => {
                let value_bit_size = self.dfg.type_of_value(value).bit_size();
                match &self.dfg[instruction] {
                    Instruction::Cast(original_value, _) => {
                        let original_bit_size = self.bits(*original_value);
                        // We might have cast e.g. `u1` to `u8` to be able to do arithmetic,
                        // in which case we want to recover the original smaller bit size;
                        // OTOH if we cast down, then we don't need the higher original size.
                        value_bit_size.min(original_bit_size)
                    }
                    Instruction::Truncate { bit_size, .. } => value_bit_size.min(*bit_size),
                    _ => value_bit_size,
                }
            }
            Value::NumericConstant { constant, .. } => constant.num_bits(),
            _ => self.dfg.type_of_value(value).bit_size(),
        }
    }

    /// Like [`Self::bits`], but also narrows the width using range-check and equality constraints
    /// that bound `value` (sound only in a single unpredicated block, where they always hold).
    pub(super) fn constrained_bits(&self, value: ValueId) -> u32 {
        if let Some(range) = self.constrained_range(value) {
            return self.dfg.type_of_value(value).bit_size().min(range.max_bits());
        }

        self.bits(value)
    }

    pub(super) fn bounds(&self, value: ValueId) -> Option<(u128, u128)> {
        self.constrained_range(value).map(|range| (range.min, range.max))
    }

    fn constrained_range(&self, value: ValueId) -> Option<Range> {
        let mut facts = Facts::default();

        for (value, _) in self.dfg.values_iter() {
            if let Some(range) = self.range(value) {
                facts.set(value, range);
            }
        }

        // Branch-local or predicated constraints cannot be used as global value bounds.
        if self.dfg.blocks.len() == 1
            && !self.dfg.instructions.iter().any(|(_, instruction)| {
                matches!(instruction, Instruction::EnableSideEffectsIf { .. })
            })
        {
            for (_, instruction) in self.dfg.instructions.iter() {
                if let Instruction::RangeCheck { value, max_bit_size, .. } = instruction
                    && let Some(max) = max_unsigned_value_for_bit_size(*max_bit_size)
                {
                    facts.refine(self.dfg, *value, Range::new(0, max));
                }
            }

            // Propagate range information in both directions until the facts stop changing.
            // Safety bound; this normally exits early once no facts change.
            for _ in 0..=self.dfg.instructions.len() {
                let mut changed = false;

                for (instruction, instruction_data) in self.dfg.instructions.iter() {
                    let result = self
                        .dfg
                        .results
                        .get(&instruction)
                        .and_then(|results| results.first())
                        .copied();

                    if let Some(result) = result
                        && let Some(range) = self.instruction_range(
                            instruction_data,
                            result,
                            RangeSource::Facts(&facts),
                        )
                    {
                        changed |= facts.refine(self.dfg, result, range);
                    }

                    changed |= self.backward(instruction_data, result, &mut facts);
                    if let Instruction::Constrain(lhs, rhs, _) = instruction_data {
                        changed |= match (facts.range(*lhs), facts.range(*rhs)) {
                            (Some(lhs_range), Some(rhs_range)) => {
                                lhs_range.intersect(rhs_range).is_some_and(|range| {
                                    // Keep bitwise OR so both sides are refined.
                                    facts.refine(self.dfg, *lhs, range)
                                        | facts.refine(self.dfg, *rhs, range)
                                })
                            }
                            (Some(range), None) => facts.refine(self.dfg, *rhs, range),
                            (None, Some(range)) => facts.refine(self.dfg, *lhs, range),
                            (None, None) => false,
                        };
                    }
                }

                if !changed {
                    break;
                }
            }
        }

        facts.range(value)
    }

    /// Compute an instruction result range from either recursive analysis or known facts.
    fn instruction_range(
        &self,
        instruction: &Instruction,
        result: ValueId,
        source: RangeSource<'_>,
    ) -> Option<Range> {
        let result_type = self.dfg.type_of_value(result);
        let value_bit_size = result_type.bit_size();

        match instruction {
            Instruction::Cast(original_value, _) => match result_type.as_ref() {
                Type::Numeric(NumericType::NativeField) => match source {
                    RangeSource::Recursive => None,
                    RangeSource::Facts(facts) => facts.range(*original_value),
                },
                Type::Numeric(NumericType::Unsigned { bit_size }) => {
                    let max = max_unsigned_value_for_bit_size(*bit_size)?;
                    let original_range =
                        source.range_or_fallback(self, *original_value, Range::new(0, max))?;
                    Some(original_range.truncate_to(max))
                }
                _ => None,
            },
            Instruction::Truncate { value: original_value, bit_size, .. } => {
                if !matches!(
                    result_type.as_ref(),
                    Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField)
                ) {
                    return None;
                }

                let max = max_unsigned_value_for_bit_size(value_bit_size.min(*bit_size))?;
                let original_range =
                    source.range_or_fallback(self, *original_value, Range::new(0, max))?;
                Some(original_range.truncate_to(max))
            }
            Instruction::Binary(binary) => {
                let ranges = self.binary_ranges(binary, value_bit_size, source);
                binary.operator.forward(ranges)
            }
            Instruction::Not(original_value) => {
                if !matches!(result_type.as_ref(), Type::Numeric(NumericType::Unsigned { .. })) {
                    return None;
                }

                let type_max = max_unsigned_value_for_bit_size(value_bit_size)?;
                let original_range =
                    source.range_or_fallback(self, *original_value, Range::new(0, type_max))?;
                Some(original_range.not(type_max))
            }
            _ => None,
        }
    }

    /// Use a known result range to tighten operand ranges where the operation is invertible enough
    /// to produce sound bounds.
    fn backward(
        &self,
        instruction: &Instruction,
        result: Option<ValueId>,
        facts: &mut Facts,
    ) -> bool {
        let Some(result) = result else {
            return false;
        };
        let Some(result_range) = facts.range(result) else {
            return false;
        };

        match instruction {
            Instruction::Cast(original_value, _) => {
                let original_type = self.dfg.type_of_value(*original_value);
                let result_type = self.dfg.type_of_value(result);
                let is_lossless_unsigned_cast = match (original_type.as_ref(), result_type.as_ref())
                {
                    (
                        Type::Numeric(NumericType::Unsigned { .. }),
                        Type::Numeric(NumericType::NativeField),
                    ) => true,
                    (
                        Type::Numeric(NumericType::Unsigned { bit_size: original_bit_size }),
                        Type::Numeric(NumericType::Unsigned { bit_size: result_bit_size }),
                    ) => original_bit_size <= result_bit_size,
                    _ => false,
                };
                if !is_lossless_unsigned_cast {
                    return false;
                }
                facts.refine(self.dfg, *original_value, result_range)
            }
            Instruction::Binary(binary) => {
                let value_bit_size = self.dfg.type_of_value(binary.lhs).bit_size();
                let Some(ranges) =
                    self.binary_ranges(binary, value_bit_size, RangeSource::Facts(facts))
                else {
                    return false;
                };

                binary.operator.backward(BinaryBack {
                    dfg: self.dfg,
                    facts,
                    lhs: binary.lhs,
                    rhs: binary.rhs,
                    result: result_range,
                    ranges,
                })
            }
            Instruction::Not(original_value) => {
                let original_type = self.dfg.type_of_value(*original_value);
                let Type::Numeric(NumericType::Unsigned { bit_size }) = original_type.as_ref()
                else {
                    return false;
                };
                let Some(type_max) = max_unsigned_value_for_bit_size(*bit_size) else {
                    return false;
                };

                facts.refine(self.dfg, *original_value, result_range.not(type_max))
            }
            _ => false,
        }
    }

    fn range(&self, value: ValueId) -> Option<Range> {
        let value_type = self.dfg.type_of_value(value);
        if !matches!(value_type.as_ref(), Type::Numeric(_)) {
            return None;
        }

        match self.dfg[value] {
            Value::NumericConstant { constant, typ } => {
                if typ.is_signed() {
                    return None;
                }

                constant.try_into_u128().map(|value| Range::new(value, value))
            }
            Value::Instruction { instruction, .. } => {
                let instruction = &self.dfg[instruction];
                match instruction {
                    Instruction::Cast(..)
                    | Instruction::Truncate { .. }
                    | Instruction::Binary(_)
                    | Instruction::Not(_) => {
                        self.instruction_range(instruction, value, RangeSource::Recursive)
                    }
                    _ => self.type_range(value),
                }
            }
            _ => self.type_range(value),
        }
    }

    fn binary_ranges(
        &self,
        binary: &Binary,
        value_bit_size: u32,
        source: RangeSource<'_>,
    ) -> Option<BinaryRanges> {
        if !self.dfg.type_of_value(binary.lhs).unwrap_numeric().is_unsigned() {
            return None;
        }

        let lhs = source.range(self, binary.lhs)?;
        let rhs = source.range(self, binary.rhs)?;
        BinaryRanges::new(value_bit_size, lhs, rhs)
    }

    fn type_range(&self, value: ValueId) -> Option<Range> {
        let typ = self.dfg.type_of_value(value);
        let Type::Numeric(NumericType::Unsigned { bit_size }) = typ.as_ref() else {
            return None;
        };

        Some(Range::new(0, max_unsigned_value_for_bit_size(*bit_size)?))
    }
}

#[derive(Default, Debug)]
struct Facts {
    ranges: HashMap<ValueId, Range>,
}

impl Facts {
    fn range(&self, value: ValueId) -> Option<Range> {
        self.ranges.get(&value).copied()
    }

    fn set(&mut self, value: ValueId, range: Range) {
        self.ranges.insert(value, range);
    }

    /// Intersect `value`'s current range with `range`.
    ///
    /// Empty refinements are ignored. They can appear when independent conservative facts cannot
    /// overlap, and inventing a replacement singleton would make later inferences unsound.
    fn refine(&mut self, dfg: &DataFlowGraph, value: ValueId, range: Range) -> bool {
        let value_type = dfg.type_of_value(value);
        let Type::Numeric(numeric_type) = value_type.as_ref() else {
            return false;
        };

        let type_max = match numeric_type {
            NumericType::Unsigned { bit_size } => max_unsigned_value_for_bit_size(*bit_size),
            NumericType::NativeField => Some(range.max),
            NumericType::Signed { .. } => return false,
        };
        let Some(type_max) = type_max else {
            return false;
        };

        let min = range.min.min(type_max);
        let max = range.max.min(type_max);
        if min > max {
            return false;
        }
        let range = Range::new(min, max);

        let Some(existing) = self.range(value) else {
            self.set(value, range);
            return true;
        };

        let Some(range) = existing.intersect(range) else {
            return false;
        };

        if range != existing {
            self.set(value, range);
            true
        } else {
            false
        }
    }
}

/// Inclusive range of possible values for an unsigned SSA value.
///
/// These ranges are deliberately conservative: if an operation can overflow or truncate, the
/// lower bound falls back to zero rather than assuming the overflowing case is unreachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Range {
    min: u128,
    max: u128,
}

impl Range {
    fn new(min: u128, max: u128) -> Self {
        debug_assert!(min <= max);
        Self { min, max }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        if min <= max { Some(Self::new(min, max)) } else { None }
    }

    fn truncate_to(self, max: u128) -> Self {
        if self.max <= max {
            self
        } else {
            // If the source can exceed the target max, truncation may wrap to any target value.
            Self::new(0, max)
        }
    }

    fn not(self, type_max: u128) -> Self {
        Self::new(type_max - self.max, type_max - self.min)
    }

    fn increasing_result(
        self,
        rhs: Self,
        type_max: u128,
        operation: impl Fn(u128, u128) -> Option<u128>,
    ) -> Self {
        let max_result = operation(self.max, rhs.max);
        let may_exceed_type = max_result.is_none_or(|result| result > type_max);
        let max = max_result.unwrap_or(type_max).min(type_max);
        let min = if may_exceed_type {
            0
        } else {
            operation(self.min, rhs.min).filter(|result| *result <= type_max).unwrap_or(0)
        };
        Self::new(min, max)
    }

    fn max_bits(self) -> u32 {
        u128::BITS - self.max.leading_zeros()
    }
}

#[derive(Clone, Copy)]
struct BinaryRanges {
    lhs: Range,
    rhs: Range,
    bit_size: u32,
    type_max: u128,
}

impl BinaryRanges {
    fn new(bit_size: u32, lhs: Range, rhs: Range) -> Option<Self> {
        Some(Self { lhs, rhs, bit_size, type_max: max_unsigned_value_for_bit_size(bit_size)? })
    }

    fn add(self) -> Range {
        self.lhs.increasing_result(self.rhs, self.type_max, u128::checked_add)
    }

    fn sub(self, unchecked: bool) -> Range {
        if unchecked {
            if self.lhs.min >= self.rhs.max {
                Range::new(self.lhs.min - self.rhs.max, self.lhs.max - self.rhs.min)
            } else {
                Range::new(0, self.type_max)
            }
        } else {
            let min = self.lhs.min.saturating_sub(self.rhs.max);
            let max = self.lhs.max.saturating_sub(self.rhs.min);
            Range::new(min, max)
        }
    }

    fn mul(self) -> Range {
        self.lhs.increasing_result(self.rhs, self.type_max, u128::checked_mul)
    }

    fn div(self) -> Range {
        let max = if self.rhs.min == 0 { self.lhs.max } else { self.lhs.max / self.rhs.min };
        let min = if self.rhs.min == 0 { 0 } else { self.lhs.min / self.rhs.max };
        Range::new(min, max)
    }

    fn modulo(self) -> Range {
        let max = if self.rhs.min == 0 {
            self.lhs.max
        } else {
            self.lhs.max.min(self.rhs.max.saturating_sub(1))
        };
        Range::new(0, max)
    }

    fn bitand(self) -> Range {
        Range::new(0, self.lhs.max.min(self.rhs.max))
    }

    fn bit_or_xor(self) -> Range {
        let max_bits = self.lhs.max_bits().max(self.rhs.max_bits());
        let max =
            max_unsigned_value_for_bit_size(max_bits).unwrap_or(self.type_max).min(self.type_max);
        Range::new(0, max)
    }

    fn shl(self) -> Range {
        if self.rhs.min == self.rhs.max && self.rhs.max < 128 {
            let shift = self.rhs.max as u32;
            let max_shifted = self.lhs.max.checked_shl(shift);
            let overflow_possible = max_shifted.is_none_or(|shifted| shifted > self.type_max);
            let max = max_shifted.unwrap_or(self.type_max).min(self.type_max);
            let min = if overflow_possible {
                0
            } else {
                self.lhs
                    .min
                    .checked_shl(shift)
                    .filter(|shifted| *shifted <= self.type_max)
                    .unwrap_or(0)
            };
            Range::new(min, max)
        } else {
            Range::new(0, self.type_max)
        }
    }

    fn shr(self) -> Range {
        if self.rhs.max < u128::from(self.bit_size) {
            let max = self.lhs.max >> self.rhs.min as u32;
            let min = self.lhs.min >> self.rhs.max as u32;
            Range::new(min, max)
        } else {
            Range::new(0, self.type_max)
        }
    }
}

struct BinaryBack<'a> {
    dfg: &'a DataFlowGraph,
    facts: &'a mut Facts,
    lhs: ValueId,
    rhs: ValueId,
    result: Range,
    ranges: BinaryRanges,
}

impl<'a> BinaryBack<'a> {
    fn add(&self, unchecked: bool) -> Option<OperandRanges> {
        if unchecked && self.result_may_wrap(u128::checked_add) {
            return None;
        }

        let lhs_max = self.result.max.saturating_sub(self.ranges.rhs.min);
        let rhs_max = self.result.max.saturating_sub(self.ranges.lhs.min);
        let lhs_min = self.result.min.saturating_sub(self.ranges.rhs.max);
        let rhs_min = self.result.min.saturating_sub(self.ranges.lhs.max);

        Some(
            self.operands()
                .tighten_lhs_bounds(lhs_min, lhs_max)
                .tighten_rhs_bounds(rhs_min, rhs_max),
        )
    }

    fn sub(&self, unchecked: bool) -> Option<OperandRanges> {
        if unchecked && self.ranges.lhs.min < self.ranges.rhs.max {
            return None;
        }

        let lhs_min =
            self.result.min.checked_add(self.ranges.rhs.min).unwrap_or(self.ranges.type_max);
        let lhs_max =
            self.result.max.checked_add(self.ranges.rhs.max).unwrap_or(self.ranges.type_max);
        let rhs_min = self.ranges.lhs.min.saturating_sub(self.result.max);
        let rhs_max = self.ranges.lhs.max.saturating_sub(self.result.min);

        Some(
            self.operands()
                .tighten_lhs_bounds(lhs_min, lhs_max)
                .tighten_rhs_bounds(rhs_min, rhs_max),
        )
    }

    fn mul(&self, unchecked: bool) -> Option<OperandRanges> {
        if unchecked && self.result_may_wrap(u128::checked_mul) {
            return None;
        }

        let mut operands = self.operands();
        if self.ranges.rhs.min > 0 {
            operands = operands.tighten_lhs_bounds(
                ceil_div(self.result.min, self.ranges.rhs.max),
                self.result.max / self.ranges.rhs.min,
            );
        }
        if self.ranges.lhs.min > 0 {
            operands = operands.tighten_rhs_bounds(
                ceil_div(self.result.min, self.ranges.lhs.max),
                self.result.max / self.ranges.lhs.min,
            );
        }
        Some(operands)
    }

    fn div(&self) -> Option<OperandRanges> {
        let mut operands = self.operands();

        if self.ranges.rhs.max > 0 {
            let lhs_max = self
                .result
                .max
                .checked_add(1)
                .and_then(|max_plus_one| max_plus_one.checked_mul(self.ranges.rhs.max))
                .and_then(|exclusive_bound| exclusive_bound.checked_sub(1))
                .unwrap_or(self.ranges.type_max)
                .min(self.ranges.type_max);
            operands = operands.tighten_lhs_bounds(0, lhs_max);
        }

        if self.ranges.rhs.min > 0 {
            let lhs_min = self
                .result
                .min
                .checked_mul(self.ranges.rhs.min)
                .unwrap_or(self.ranges.type_max)
                .min(self.ranges.type_max);
            operands = operands.tighten_lhs_bounds(lhs_min, self.ranges.type_max);
        }

        Some(operands)
    }

    fn bitor(&self) -> Option<OperandRanges> {
        Some(
            self.operands()
                .tighten_lhs(Range::new(0, self.result.max))
                .tighten_rhs(Range::new(0, self.result.max)),
        )
    }

    fn shl(&self) -> Option<OperandRanges> {
        if self.ranges.rhs.min != self.ranges.rhs.max || self.ranges.rhs.max >= 128 {
            return None;
        }

        let shift = self.ranges.rhs.max as u32;
        if !self.ranges.lhs.max.checked_shl(shift).is_some_and(|max| max <= self.ranges.type_max) {
            return None;
        }

        Some(self.operands().tighten_lhs(Range::new(0, self.result.max >> shift)))
    }

    fn apply(self, operands: Option<OperandRanges>) -> bool {
        let Some(operands) = operands else {
            return false;
        };

        self.facts.refine(self.dfg, self.lhs, operands.lhs)
            | self.facts.refine(self.dfg, self.rhs, operands.rhs)
    }

    fn operands(&self) -> OperandRanges {
        OperandRanges { lhs: self.ranges.lhs, rhs: self.ranges.rhs }
    }

    fn result_may_wrap(&self, operation: impl FnOnce(u128, u128) -> Option<u128>) -> bool {
        operation(self.ranges.lhs.max, self.ranges.rhs.max)
            .is_none_or(|result| result > self.ranges.type_max)
    }
}

struct OperandRanges {
    lhs: Range,
    rhs: Range,
}

impl OperandRanges {
    // Empty intersections are ignored to match `Facts::refine`.
    fn tighten_lhs_bounds(self, min: u128, max: u128) -> Self {
        if min <= max { self.tighten_lhs(Range::new(min, max)) } else { self }
    }

    fn tighten_rhs_bounds(self, min: u128, max: u128) -> Self {
        if min <= max { self.tighten_rhs(Range::new(min, max)) } else { self }
    }

    fn tighten_lhs(mut self, range: Range) -> Self {
        if let Some(range) = self.lhs.intersect(range) {
            self.lhs = range;
        }
        self
    }

    fn tighten_rhs(mut self, range: Range) -> Self {
        if let Some(range) = self.rhs.intersect(range) {
            self.rhs = range;
        }
        self
    }
}

impl BinaryOp {
    fn forward(self, ranges: Option<BinaryRanges>) -> Option<Range> {
        match self {
            BinaryOp::Eq | BinaryOp::Lt => Some(Range::new(0, 1)),
            BinaryOp::Add { .. } => Some(ranges?.add()),
            BinaryOp::Sub { unchecked } => Some(ranges?.sub(unchecked)),
            BinaryOp::Mul { .. } => Some(ranges?.mul()),
            BinaryOp::Div => Some(ranges?.div()),
            BinaryOp::Mod => Some(ranges?.modulo()),
            BinaryOp::And => Some(ranges?.bitand()),
            BinaryOp::Or | BinaryOp::Xor => Some(ranges?.bit_or_xor()),
            BinaryOp::Shl => Some(ranges?.shl()),
            BinaryOp::Shr => Some(ranges?.shr()),
        }
    }

    fn backward(self, back: BinaryBack<'_>) -> bool {
        let operands = match self {
            BinaryOp::Add { unchecked } => back.add(unchecked),
            BinaryOp::Sub { unchecked } => back.sub(unchecked),
            BinaryOp::Mul { unchecked } => back.mul(unchecked),
            BinaryOp::Div => back.div(),
            BinaryOp::Or => back.bitor(),
            BinaryOp::Shl => back.shl(),
            BinaryOp::Mod
            | BinaryOp::And
            | BinaryOp::Xor
            | BinaryOp::Shr
            | BinaryOp::Eq
            | BinaryOp::Lt => None,
        };
        back.apply(operands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U8_BITS: u32 = 8;

    fn u8_ranges(lhs: Range, rhs: Range) -> BinaryRanges {
        BinaryRanges::new(U8_BITS, lhs, rhs).unwrap()
    }

    fn apply_u8_back(
        lhs_range: Range,
        rhs_range: Range,
        result: Range,
        operator: BinaryOp,
    ) -> (bool, Range, Range) {
        let ranges = u8_ranges(lhs_range, rhs_range);
        let mut dfg = DataFlowGraph::default();
        let block = dfg.make_block();
        let lhs = dfg.add_block_parameter(block, Type::unsigned(U8_BITS));
        let rhs = dfg.add_block_parameter(block, Type::unsigned(U8_BITS));

        let mut facts = Facts::default();
        facts.set(lhs, lhs_range);
        facts.set(rhs, rhs_range);

        let changed = {
            let back = BinaryBack { dfg: &dfg, facts: &mut facts, lhs, rhs, result, ranges };
            operator.backward(back)
        };

        (changed, facts.range(lhs).unwrap(), facts.range(rhs).unwrap())
    }

    #[test]
    fn range_not_inverts_bounds_within_type_max() {
        assert_eq!(Range::new(3, 7).not(15), Range::new(8, 12));
    }

    #[test]
    fn range_truncate_wraps_to_full_target_width() {
        assert_eq!(Range::new(0, 10).truncate_to(15), Range::new(0, 10));
        assert_eq!(Range::new(3, 20).truncate_to(15), Range::new(0, 15));
    }

    #[test]
    fn range_intersect_returns_overlap_or_none() {
        assert_eq!(Range::new(2, 8).intersect(Range::new(5, 10)), Some(Range::new(5, 8)));
        assert_eq!(Range::new(2, 8).intersect(Range::new(9, 10)), None);
    }

    #[test]
    fn range_increasing_result_uses_full_range_on_overflow() {
        let range =
            Range::new(200, 254).increasing_result(Range::new(2, 2), 255, u128::checked_add);
        assert_eq!(range, Range::new(0, 255));
    }

    #[test]
    fn recursive_source_uses_cast_fallback_but_facts_source_requires_a_known_range() {
        let mut dfg = DataFlowGraph::default();
        let block = dfg.make_block();
        let original = dfg.add_block_parameter(block, Type::signed(16));
        let result = dfg.add_block_parameter(block, Type::unsigned(8));
        let facts = Facts::default();
        let analysis = Analysis::new(&dfg);

        let recursive_range = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::unsigned(8)),
            result,
            RangeSource::Recursive,
        );
        let facts_range = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::unsigned(8)),
            result,
            RangeSource::Facts(&facts),
        );

        assert_eq!(recursive_range, Some(Range::new(0, 255)));
        assert_eq!(facts_range, None);
    }

    #[test]
    fn field_casts_only_propagate_ranges_from_facts() {
        let mut dfg = DataFlowGraph::default();
        let block = dfg.make_block();
        let original = dfg.add_block_parameter(block, Type::unsigned(8));
        let result = dfg.add_block_parameter(block, Type::field());
        let mut facts = Facts::default();
        facts.set(original, Range::new(3, 7));
        let analysis = Analysis::new(&dfg);

        let recursive_range = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::NativeField),
            result,
            RangeSource::Recursive,
        );
        let facts_range = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::NativeField),
            result,
            RangeSource::Facts(&facts),
        );

        assert_eq!(recursive_range, None);
        assert_eq!(facts_range, Some(Range::new(3, 7)));
    }

    #[test]
    fn binary_ranges_add_falls_back_when_sum_may_wrap() {
        let ranges = u8_ranges(Range::new(250, 255), Range::new(1, 10));

        assert_eq!(ranges.add(), Range::new(0, 255));
    }

    #[test]
    fn binary_ranges_mul_falls_back_when_product_may_wrap() {
        let ranges = u8_ranges(Range::new(100, 200), Range::new(2, 3));

        assert_eq!(ranges.mul(), Range::new(0, 255));
    }

    #[test]
    fn binary_ranges_sub_distinguishes_checked_and_unchecked_wrap() {
        let ranges = u8_ranges(Range::new(5, 10), Range::new(7, 8));

        assert_eq!(ranges.sub(false), Range::new(0, 3));
        assert_eq!(ranges.sub(true), Range::new(0, 255));
    }

    #[test]
    fn binary_ranges_div_uses_lhs_max_when_rhs_can_be_zero() {
        let ranges = u8_ranges(Range::new(10, 20), Range::new(0, 5));

        assert_eq!(ranges.div(), Range::new(0, 20));
    }

    #[test]
    fn binary_ranges_modulo_uses_lhs_max_when_rhs_can_be_zero() {
        let ranges = u8_ranges(Range::new(10, 20), Range::new(0, 5));

        assert_eq!(ranges.modulo(), Range::new(0, 20));
    }

    #[test]
    fn binary_back_add_refines_both_operands() {
        let (changed, lhs, rhs) = apply_u8_back(
            Range::new(0, 255),
            Range::new(0, 255),
            Range::new(0, 15),
            BinaryOp::Add { unchecked: false },
        );

        assert!(changed);
        assert_eq!(lhs, Range::new(0, 15));
        assert_eq!(rhs, Range::new(0, 15));
    }

    #[test]
    fn binary_back_sub_refines_checked_operands() {
        let (changed, lhs, rhs) = apply_u8_back(
            Range::new(0, 255),
            Range::new(0, 255),
            Range::new(10, 20),
            BinaryOp::Sub { unchecked: false },
        );

        assert!(changed);
        assert_eq!(lhs, Range::new(10, 255));
        assert_eq!(rhs, Range::new(0, 245));
    }

    #[test]
    fn binary_back_mul_refines_positive_operands() {
        let (changed, lhs, rhs) = apply_u8_back(
            Range::new(1, 255),
            Range::new(1, 255),
            Range::new(4, 20),
            BinaryOp::Mul { unchecked: false },
        );

        assert!(changed);
        assert_eq!(lhs, Range::new(1, 20));
        assert_eq!(rhs, Range::new(1, 20));
    }

    #[test]
    fn binary_back_div_refines_lhs_from_nonzero_rhs() {
        let (changed, lhs, rhs) =
            apply_u8_back(Range::new(0, 255), Range::new(2, 10), Range::new(3, 4), BinaryOp::Div);

        assert!(changed);
        assert_eq!(lhs, Range::new(6, 49));
        assert_eq!(rhs, Range::new(2, 10));
    }

    #[test]
    fn binary_back_shl_refines_lhs_for_fixed_shift() {
        let (changed, lhs, rhs) =
            apply_u8_back(Range::new(0, 63), Range::new(2, 2), Range::new(0, 31), BinaryOp::Shl);

        assert!(changed);
        assert_eq!(lhs, Range::new(0, 7));
        assert_eq!(rhs, Range::new(2, 2));
    }
}
