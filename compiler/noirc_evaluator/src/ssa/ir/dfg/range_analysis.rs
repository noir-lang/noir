use acvm::AcirField;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::{
    instruction::{Binary, BinaryOp, Instruction, InstructionId},
    types::{NumericType, Type},
    value::{Value, ValueId},
};

use super::DataFlowGraph;

/// Return the maximum unsigned value representable by `bit_size` bits.
pub(crate) fn max_unsigned_value_for_bit_size(bit_size: u32) -> Option<u128> {
    match bit_size {
        0 => Some(0),
        1..=127 => Some((1u128 << bit_size) - 1),
        128 => Some(u128::MAX),
        _ => None,
    }
}

fn u128_num_bits(value: u128) -> u32 {
    u128::BITS - value.leading_zeros()
}

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    debug_assert!(denominator > 0);

    let quotient = numerator / denominator;
    quotient + u128::from(numerator % denominator != 0)
}

/// Computes conservative unsigned value ranges for SSA values.
pub(super) struct Analysis<'dfg> {
    dfg: &'dfg DataFlowGraph,
}

impl<'dfg> Analysis<'dfg> {
    pub(super) fn new(dfg: &'dfg DataFlowGraph) -> Self {
        Self { dfg }
    }

    /// Returns the maximum possible number of bits that `value` can potentially be.
    ///
    /// Should `value` be a numeric constant then this function will return the exact number of
    /// bits required, otherwise it will return the minimum number of bits based on type information.
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
                    Instruction::Binary(binary) => {
                        if !self.dfg.type_of_value(binary.lhs).unwrap_numeric().is_unsigned() {
                            return value_bit_size;
                        }

                        let lhs_bits = self.bits(binary.lhs);
                        let rhs_bits = self.bits(binary.rhs);
                        binary.operator.max_bits(lhs_bits, rhs_bits, value_bit_size)
                    }
                    _ => value_bit_size,
                }
            }
            Value::NumericConstant { constant, .. } => constant.num_bits(),
            _ => self.dfg.type_of_value(value).bit_size(),
        }
    }

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
        let mut facts = self.seed_facts();
        self.apply_unconditional_constraints(&mut facts);
        facts.range(value)
    }

    fn seed_facts(&self) -> Facts {
        let mut facts = Facts::default();

        for (value, _) in self.dfg.values_iter() {
            if let Some(range) = self.range(value) {
                facts.set(value, range);
            }
        }

        facts
    }

    /// Apply constraints that are guaranteed to hold whenever this function executes.
    ///
    /// The fixed-point loop propagates range information in both directions through instructions:
    /// result ranges refine operand ranges, operand ranges refine result ranges, and equality
    /// constraints intersect the ranges of both sides.
    fn apply_unconditional_constraints(&self, facts: &mut Facts) {
        // Branch-local or predicated constraints cannot be used as global value bounds.
        if self.dfg.blocks.len() != 1 || self.has_side_effect_predicates() {
            return;
        }

        self.seed_range_checks(facts);
        self.propagate(facts);
    }

    fn seed_range_checks(&self, facts: &mut Facts) {
        for (_, instruction) in self.dfg.instructions.iter() {
            if let Instruction::RangeCheck { value, max_bit_size, .. } = instruction {
                let Some(max) = max_unsigned_value_for_bit_size(*max_bit_size) else {
                    continue;
                };
                facts.refine(self.dfg, *value, Range::new(0, max));
            }
        }
    }

    fn propagate(&self, facts: &mut Facts) {
        for _ in 0..=self.dfg.instructions.len() {
            let mut changed = false;

            for (instruction, instruction_data) in self.dfg.instructions.iter() {
                changed |= self.visit_instruction(instruction, instruction_data, facts);
            }

            if !changed {
                break;
            }
        }
    }

    fn has_side_effect_predicates(&self) -> bool {
        self.dfg
            .instructions
            .iter()
            .any(|(_, instruction)| matches!(instruction, Instruction::EnableSideEffectsIf { .. }))
    }

    fn visit_instruction(
        &self,
        instruction: InstructionId,
        instruction_data: &Instruction,
        facts: &mut Facts,
    ) -> bool {
        let result =
            self.dfg.results.get(&instruction).and_then(|results| results.first()).copied();
        let mut changed = false;

        if let Some(result) = result {
            if let Some(range) = self.forward(instruction_data, result, facts) {
                changed |= facts.refine(self.dfg, result, range);
            }
        }

        changed |= self.backward(instruction_data, result, facts);
        changed |= self.equality(instruction_data, facts);

        changed
    }

    /// Compute an instruction result range from already-known operand ranges.
    fn forward(&self, instruction: &Instruction, result: ValueId, facts: &Facts) -> Option<Range> {
        let value_bit_size = self.dfg.type_of_value(result).bit_size();

        match instruction {
            Instruction::Cast(original_value, _) => {
                if !matches!(
                    self.dfg.type_of_value(result).as_ref(),
                    Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField)
                ) {
                    return None;
                }

                let original_range = facts.range(*original_value)?;
                match self.dfg.type_of_value(result).as_ref() {
                    Type::Numeric(NumericType::NativeField) => Some(original_range),
                    Type::Numeric(NumericType::Unsigned { bit_size }) => {
                        let max = max_unsigned_value_for_bit_size(*bit_size)?;
                        Some(original_range.truncate_to(max))
                    }
                    _ => None,
                }
            }
            Instruction::Truncate { value: original_value, bit_size, .. } => {
                if !matches!(
                    self.dfg.type_of_value(result).as_ref(),
                    Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField)
                ) {
                    return None;
                }

                let max = max_unsigned_value_for_bit_size(value_bit_size.min(*bit_size))?;
                let original_range = facts.range(*original_value)?;
                Some(original_range.truncate_to(max))
            }
            Instruction::Binary(binary) => self.forward_binary(binary, value_bit_size, facts),
            Instruction::Not(original_value) => {
                if !matches!(
                    self.dfg.type_of_value(result).as_ref(),
                    Type::Numeric(NumericType::Unsigned { .. })
                ) {
                    return None;
                }

                let type_max = max_unsigned_value_for_bit_size(value_bit_size)?;
                let original_range = facts.range(*original_value)?;
                Some(original_range.not(type_max))
            }
            _ => None,
        }
    }

    fn forward_binary(&self, binary: &Binary, value_bit_size: u32, facts: &Facts) -> Option<Range> {
        let ranges = self.binary_ranges(binary, value_bit_size, facts);
        binary.operator.forward(ranges)
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
                if !self.is_lossless_unsigned_cast(*original_value, result) {
                    return false;
                }
                facts.refine(self.dfg, *original_value, result_range)
            }
            Instruction::Binary(binary) => self.backward_binary(binary, result_range, facts),
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

    fn backward_binary(&self, binary: &Binary, result: Range, facts: &mut Facts) -> bool {
        let value_bit_size = self.dfg.type_of_value(binary.lhs).bit_size();
        let Some(ranges) = self.binary_ranges(binary, value_bit_size, facts) else {
            return false;
        };

        binary.operator.backward(BinaryBack { analysis: self, facts, binary, result, ranges })
    }

    fn equality(&self, instruction: &Instruction, facts: &mut Facts) -> bool {
        let Instruction::Constrain(lhs, rhs, _) = instruction else {
            return false;
        };

        self.propagate_equality(*lhs, *rhs, facts)
    }

    fn propagate_equality(&self, lhs: ValueId, rhs: ValueId, facts: &mut Facts) -> bool {
        let lhs_range = facts.range(lhs);
        let rhs_range = facts.range(rhs);

        match (lhs_range, rhs_range) {
            (Some(lhs_range), Some(rhs_range)) => {
                let Some(range) = lhs_range.intersect(rhs_range) else {
                    return false;
                };

                facts.refine(self.dfg, lhs, range) | facts.refine(self.dfg, rhs, range)
            }
            (Some(range), None) => facts.refine(self.dfg, rhs, range),
            (None, Some(range)) => facts.refine(self.dfg, lhs, range),
            (None, None) => false,
        }
    }

    fn is_lossless_unsigned_cast(&self, original_value: ValueId, result: ValueId) -> bool {
        let original_type = self.dfg.type_of_value(original_value);
        let result_type = self.dfg.type_of_value(result);

        match (original_type.as_ref(), result_type.as_ref()) {
            (
                Type::Numeric(NumericType::Unsigned { .. }),
                Type::Numeric(NumericType::NativeField),
            ) => true,
            (
                Type::Numeric(NumericType::Unsigned { bit_size: original_bit_size }),
                Type::Numeric(NumericType::Unsigned { bit_size: result_bit_size }),
            ) => original_bit_size <= result_bit_size,
            _ => false,
        }
    }

    fn range(&self, value: ValueId) -> Option<Range> {
        let value_type = self.dfg.type_of_value(value);
        if !matches!(value_type.as_ref(), Type::Numeric(_)) {
            return None;
        }
        let value_bit_size = value_type.bit_size();

        match self.dfg[value] {
            Value::NumericConstant { constant, typ } => {
                if typ.is_signed() {
                    return None;
                }

                constant.try_into_u128().map(|value| Range::new(value, value))
            }
            Value::Instruction { instruction, .. } => match &self.dfg[instruction] {
                Instruction::Cast(original_value, _) => {
                    if !matches!(
                        self.dfg.type_of_value(value).as_ref(),
                        Type::Numeric(NumericType::Unsigned { .. })
                    ) {
                        return None;
                    }

                    let max = max_unsigned_value_for_bit_size(value_bit_size)?;
                    let original_range = self.range(*original_value);

                    Some(original_range.map_or(Range::new(0, max), |range| range.truncate_to(max)))
                }
                Instruction::Truncate { value: original_value, bit_size, .. } => {
                    if !matches!(
                        self.dfg.type_of_value(value).as_ref(),
                        Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField)
                    ) {
                        return None;
                    }

                    let max = max_unsigned_value_for_bit_size(value_bit_size.min(*bit_size))?;
                    let original_range = self.range(*original_value);

                    Some(original_range.map_or(Range::new(0, max), |range| range.truncate_to(max)))
                }
                Instruction::Binary(binary) => self.local_binary_range(binary, value_bit_size),
                Instruction::Not(original_value) => {
                    if !matches!(
                        self.dfg.type_of_value(value).as_ref(),
                        Type::Numeric(NumericType::Unsigned { .. })
                    ) {
                        return None;
                    }

                    let type_max = max_unsigned_value_for_bit_size(value_bit_size)?;
                    let original_range =
                        self.range(*original_value).unwrap_or_else(|| Range::new(0, type_max));
                    Some(original_range.not(type_max))
                }
                _ => self.type_range(value),
            },
            _ => self.type_range(value),
        }
    }

    fn local_binary_range(&self, binary: &Binary, value_bit_size: u32) -> Option<Range> {
        let ranges = self.local_binary_ranges(binary, value_bit_size);
        binary.operator.forward(ranges)
    }

    fn local_binary_ranges(&self, binary: &Binary, value_bit_size: u32) -> Option<BinaryRanges> {
        if !self.dfg.type_of_value(binary.lhs).unwrap_numeric().is_unsigned() {
            return None;
        }

        let lhs = self.range(binary.lhs)?;
        let rhs = self.range(binary.rhs)?;
        BinaryRanges::new(value_bit_size, lhs, rhs)
    }

    fn binary_ranges(
        &self,
        binary: &Binary,
        value_bit_size: u32,
        facts: &Facts,
    ) -> Option<BinaryRanges> {
        if !self.dfg.type_of_value(binary.lhs).unwrap_numeric().is_unsigned() {
            return None;
        }

        let lhs = facts.range(binary.lhs)?;
        let rhs = facts.range(binary.rhs)?;
        BinaryRanges::new(value_bit_size, lhs, rhs)
    }

    fn type_range(&self, value: ValueId) -> Option<Range> {
        let typ = self.dfg.type_of_value(value);
        let Type::Numeric(NumericType::Unsigned { bit_size }) = typ.as_ref() else {
            return None;
        };

        Range::for_bits(*bit_size)
    }
}

#[derive(Default)]
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
        self.refine_bounds(dfg, value, range.min, range.max)
    }

    fn refine_bounds(&mut self, dfg: &DataFlowGraph, value: ValueId, min: u128, max: u128) -> bool {
        let value_type = dfg.type_of_value(value);
        let Type::Numeric(numeric_type) = value_type.as_ref() else {
            return false;
        };

        if numeric_type.is_signed() {
            return false;
        }

        let type_max = match numeric_type {
            NumericType::Unsigned { bit_size } => max_unsigned_value_for_bit_size(*bit_size),
            NumericType::NativeField => Some(max),
            NumericType::Signed { .. } => None,
        };
        let Some(type_max) = type_max else {
            return false;
        };

        let min = min.min(type_max);
        let max = max.min(type_max);
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
#[derive(Clone, Copy, PartialEq, Eq)]
struct Range {
    min: u128,
    max: u128,
}

impl Range {
    fn new(min: u128, max: u128) -> Self {
        debug_assert!(min <= max);
        Self { min, max }
    }

    fn bool() -> Self {
        Self::new(0, 1)
    }

    fn for_bits(bit_size: u32) -> Option<Self> {
        Some(Self::new(0, max_unsigned_value_for_bit_size(bit_size)?))
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

    fn max_result_fits(
        self,
        rhs: Self,
        type_max: u128,
        operation: impl FnOnce(u128, u128) -> Option<u128>,
    ) -> bool {
        operation(self.max, rhs.max).is_some_and(|result| result <= type_max)
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
        u128_num_bits(self.max)
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
        self.lhs.increasing_result(self.rhs, self.type_max, |lhs, rhs| lhs.checked_add(rhs))
    }

    fn sub(self, unchecked: bool) -> Range {
        if unchecked {
            if self.lhs.min >= self.rhs.max {
                Range::new(self.lhs.min - self.rhs.max, self.lhs.max - self.rhs.min)
            } else {
                Range::new(0, self.type_max)
            }
        } else {
            let min = self.lhs.min.checked_sub(self.rhs.max).unwrap_or(0);
            let max = self.lhs.max.saturating_sub(self.rhs.min);
            Range::new(min, max)
        }
    }

    fn mul(self) -> Range {
        self.lhs.increasing_result(self.rhs, self.type_max, |lhs, rhs| lhs.checked_mul(rhs))
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

struct BinaryBack<'a, 'dfg> {
    analysis: &'a Analysis<'dfg>,
    facts: &'a mut Facts,
    binary: &'a Binary,
    result: Range,
    ranges: BinaryRanges,
}

impl<'a, 'dfg> BinaryBack<'a, 'dfg> {
    fn add(&mut self, unchecked: bool) -> bool {
        if unchecked
            && !self.ranges.lhs.max_result_fits(
                self.ranges.rhs,
                self.ranges.type_max,
                |lhs, rhs| lhs.checked_add(rhs),
            )
        {
            return false;
        }

        let lhs_max = self.result.max.saturating_sub(self.ranges.rhs.min);
        let rhs_max = self.result.max.saturating_sub(self.ranges.lhs.min);
        let lhs_min = self.result.min.saturating_sub(self.ranges.rhs.max);
        let rhs_min = self.result.min.saturating_sub(self.ranges.lhs.max);

        self.refine_lhs(lhs_min, lhs_max) | self.refine_rhs(rhs_min, rhs_max)
    }

    fn sub(&mut self, unchecked: bool) -> bool {
        if unchecked && self.ranges.lhs.min < self.ranges.rhs.max {
            return false;
        }

        let lhs_min =
            self.result.min.checked_add(self.ranges.rhs.min).unwrap_or(self.ranges.type_max);
        let lhs_max =
            self.result.max.checked_add(self.ranges.rhs.max).unwrap_or(self.ranges.type_max);
        let rhs_min = self.ranges.lhs.min.saturating_sub(self.result.max);
        let rhs_max = self.ranges.lhs.max.saturating_sub(self.result.min);

        self.refine_lhs(lhs_min, lhs_max) | self.refine_rhs(rhs_min, rhs_max)
    }

    fn mul(&mut self, unchecked: bool) -> bool {
        if unchecked
            && !self.ranges.lhs.max_result_fits(
                self.ranges.rhs,
                self.ranges.type_max,
                |lhs, rhs| lhs.checked_mul(rhs),
            )
        {
            return false;
        }

        let mut changed = false;
        if self.ranges.rhs.min > 0 {
            changed |= self.refine_lhs(
                ceil_div(self.result.min, self.ranges.rhs.max),
                self.result.max / self.ranges.rhs.min,
            );
        }
        if self.ranges.lhs.min > 0 {
            changed |= self.refine_rhs(
                ceil_div(self.result.min, self.ranges.lhs.max),
                self.result.max / self.ranges.lhs.min,
            );
        }
        changed
    }

    fn div(&mut self) -> bool {
        let mut changed = false;

        if self.ranges.rhs.max > 0 {
            let lhs_max = self
                .result
                .max
                .checked_add(1)
                .and_then(|max_plus_one| max_plus_one.checked_mul(self.ranges.rhs.max))
                .and_then(|exclusive_bound| exclusive_bound.checked_sub(1))
                .unwrap_or(self.ranges.type_max)
                .min(self.ranges.type_max);
            changed |= self.refine_lhs(0, lhs_max);
        }

        if self.ranges.rhs.min > 0 {
            let lhs_min = self
                .result
                .min
                .checked_mul(self.ranges.rhs.min)
                .unwrap_or(self.ranges.type_max)
                .min(self.ranges.type_max);
            changed |= self.refine_lhs(lhs_min, self.ranges.type_max);
        }

        changed
    }

    fn bitor(&mut self) -> bool {
        self.refine_lhs(0, self.result.max) | self.refine_rhs(0, self.result.max)
    }

    fn shl(&mut self) -> bool {
        if self.ranges.rhs.min != self.ranges.rhs.max || self.ranges.rhs.max >= 128 {
            return false;
        }

        let shift = self.ranges.rhs.max as u32;
        if !self.ranges.lhs.max.checked_shl(shift).is_some_and(|max| max <= self.ranges.type_max) {
            return false;
        }

        self.refine_lhs(0, self.result.max >> shift)
    }

    fn refine_lhs(&mut self, min: u128, max: u128) -> bool {
        self.facts.refine_bounds(self.analysis.dfg, self.binary.lhs, min, max)
    }

    fn refine_rhs(&mut self, min: u128, max: u128) -> bool {
        self.facts.refine_bounds(self.analysis.dfg, self.binary.rhs, min, max)
    }
}

impl BinaryOp {
    fn max_bits(self, lhs_bits: u32, rhs_bits: u32, value_bit_size: u32) -> u32 {
        let max_bits = match self {
            BinaryOp::Add { .. } => lhs_bits.max(rhs_bits).saturating_add(1),
            BinaryOp::Sub { unchecked: false } => lhs_bits,
            BinaryOp::Sub { unchecked: true } => value_bit_size,
            BinaryOp::Mul { .. } => lhs_bits.saturating_add(rhs_bits),
            BinaryOp::Div => lhs_bits,
            BinaryOp::Mod => rhs_bits,
            BinaryOp::Eq | BinaryOp::Lt => 1,
            BinaryOp::And => lhs_bits.min(rhs_bits),
            BinaryOp::Or | BinaryOp::Xor => lhs_bits.max(rhs_bits),
            BinaryOp::Shl | BinaryOp::Shr => value_bit_size,
        };

        value_bit_size.min(max_bits)
    }

    fn forward(self, ranges: Option<BinaryRanges>) -> Option<Range> {
        match self {
            BinaryOp::Eq | BinaryOp::Lt => Some(Range::bool()),
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

    fn backward(self, mut back: BinaryBack<'_, '_>) -> bool {
        match self {
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
            | BinaryOp::Lt => false,
        }
    }
}
