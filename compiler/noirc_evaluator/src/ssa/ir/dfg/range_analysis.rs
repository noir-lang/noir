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
    instruction::{
        Binary, BinaryOp, Instruction, InstructionId,
        binary::try_convert_field_element_to_signed_integer,
    },
    types::{NumericType, Type, max_unsigned_value_for_bit_size},
    value::{Value, ValueId},
};

use super::DataFlowGraph;

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    debug_assert!(denominator > 0);

    let quotient = numerator / denominator;
    quotient + u128::from(numerator % denominator != 0)
}

fn sign_bit(bit_size: u32) -> Option<u128> {
    match bit_size {
        1..=128 => Some(1u128 << (bit_size - 1)),
        _ => None,
    }
}

fn signed_constant_value(constant: acvm::FieldElement, bit_size: u32) -> Option<i128> {
    if bit_size == 128 {
        constant.try_into_i128().or_else(|| constant.try_into_u128().map(|value| value as i128))
    } else {
        try_convert_field_element_to_signed_integer(constant, bit_size)
    }
}

fn signed_to_twos_complement(value: i128, bit_size: u32) -> Option<u128> {
    if value >= 0 {
        return u128::try_from(value).ok();
    }

    if bit_size == 128 {
        Some(value as u128)
    } else {
        Some((1u128 << bit_size) - value.unsigned_abs())
    }
}

fn unsigned_to_signed(value: u128, bit_size: u32) -> Option<i128> {
    if bit_size == 128 {
        Some(value as i128)
    } else if value < sign_bit(bit_size)? {
        i128::try_from(value).ok()
    } else {
        let magnitude = (1u128 << bit_size) - value;
        i128::try_from(magnitude).ok().map(|value| -value)
    }
}

/// Computes conservative numeric value ranges for SSA values.
pub(super) struct Analysis<'dfg> {
    dfg: &'dfg DataFlowGraph,
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
        self.value_bits(value, &self.infer_facts(false))
    }

    /// Like [`Self::bits`], but also narrows the width using range-check and equality constraints
    /// that bound `value` (sound only in a single unpredicated block, where they always hold).
    pub(super) fn constrained_bits(&self, value: ValueId) -> u32 {
        self.value_bits(value, &self.infer_facts(true))
    }

    pub(super) fn bounds(&self, value: ValueId) -> Option<(u128, u128)> {
        self.infer_facts(true)
            .range(value)
            .and_then(ValueRange::into_unsigned)
            .map(|range| (range.min, range.max))
    }

    fn value_bits(&self, value: ValueId, facts: &Facts) -> u32 {
        if let Some(range) = facts.range(value) {
            let value_bit_size = self.dfg.type_of_value(value).bit_size();
            value_bit_size.min(range.max_bits(value_bit_size))
        } else {
            self.dfg.type_of_value(value).bit_size()
        }
    }

    fn infer_facts(&self, include_global_constraints: bool) -> Facts {
        let mut facts = Facts::default();

        for (value, _) in self.dfg.values_iter() {
            if let Some(range) = self.initial_range(value) {
                facts.set(value, range);
            }
        }

        let use_global_constraints =
            include_global_constraints && self.can_use_global_constraints();

        // Safety cap for malformed fixed points; real propagation exits when no range narrows.
        for _ in 0..=self.dfg.instructions.len() {
            let mut changed = false;
            for (instruction, data) in self.dfg.instructions.iter() {
                changed |=
                    self.apply_instruction(instruction, data, &mut facts, use_global_constraints);
            }
            if !changed {
                break;
            }
        }

        facts
    }

    fn can_use_global_constraints(&self) -> bool {
        // Branch-local or predicated constraints cannot be used as global value bounds.
        self.dfg.blocks.len() == 1
            && !self.dfg.instructions.iter().any(|(_, instruction)| {
                matches!(instruction, Instruction::EnableSideEffectsIf { .. })
            })
    }

    fn apply_instruction(
        &self,
        instruction: InstructionId,
        instruction_data: &Instruction,
        facts: &mut Facts,
        include_global_constraints: bool,
    ) -> bool {
        let result =
            self.dfg.results.get(&instruction).and_then(|results| results.first()).copied();
        let mut changed = false;

        if let Some(result) = result
            && let Some(range) =
                self.instruction_range(instruction_data, result, facts, include_global_constraints)
        {
            changed |= self.refine(facts, result, range);
        }

        changed |= self.backward(instruction_data, result, facts);

        if include_global_constraints {
            changed |= self.apply_global_constraint(instruction_data, facts);
        }

        changed
    }

    fn refine(&self, facts: &mut Facts, value: ValueId, range: ValueRange) -> bool {
        facts.refine(self.dfg, value, range)
    }

    fn refine_pair(
        &self,
        facts: &mut Facts,
        lhs: (ValueId, ValueRange),
        rhs: (ValueId, ValueRange),
    ) -> bool {
        let lhs_changed = self.refine(facts, lhs.0, lhs.1);
        let rhs_changed = self.refine(facts, rhs.0, rhs.1);
        lhs_changed || rhs_changed
    }

    fn apply_global_constraint(&self, instruction: &Instruction, facts: &mut Facts) -> bool {
        match instruction {
            Instruction::RangeCheck { value, max_bit_size, .. } => {
                let Some(max) = max_unsigned_value_for_bit_size(*max_bit_size) else {
                    return false;
                };
                let range = match self.dfg.type_of_value(*value).as_ref() {
                    Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField) => {
                        Some(ValueRange::Unsigned(Range::new(0, max)))
                    }
                    Type::Numeric(NumericType::Signed { bit_size }) if max_bit_size < bit_size => {
                        i128::try_from(max)
                            .ok()
                            .map(|max| ValueRange::Signed(SignedRange::new(0, max)))
                    }
                    Type::Numeric(NumericType::Signed { bit_size }) => {
                        SignedRange::for_bit_size(*bit_size).map(ValueRange::Signed)
                    }
                    _ => None,
                };
                range.is_some_and(|range| self.refine(facts, *value, range))
            }
            Instruction::Constrain(lhs, rhs, _) => match (facts.range(*lhs), facts.range(*rhs)) {
                (Some(lhs_range), Some(rhs_range)) => {
                    if let Some(range) = lhs_range.intersect(rhs_range) {
                        self.refine_pair(facts, (*lhs, range), (*rhs, range))
                    } else {
                        false
                    }
                }
                (Some(range), None) => self.refine(facts, *rhs, range),
                (None, Some(range)) => self.refine(facts, *lhs, range),
                (None, None) => false,
            },
            _ => false,
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
                let is_lossless_cast = match (original_type.as_ref(), result_type.as_ref()) {
                    (
                        Type::Numeric(NumericType::Unsigned { .. }),
                        Type::Numeric(NumericType::NativeField),
                    ) => true,
                    (
                        Type::Numeric(NumericType::Unsigned { bit_size: original_bit_size }),
                        Type::Numeric(NumericType::Unsigned { bit_size: result_bit_size }),
                    ) => original_bit_size <= result_bit_size,
                    (
                        Type::Numeric(NumericType::Signed { bit_size: original_bit_size }),
                        Type::Numeric(NumericType::Signed { bit_size: result_bit_size }),
                    ) => original_bit_size <= result_bit_size,
                    _ => false,
                };
                if is_lossless_cast {
                    self.refine(facts, *original_value, result_range)
                } else {
                    false
                }
            }
            Instruction::Binary(binary) => {
                let value_bit_size = self.dfg.type_of_value(binary.lhs).bit_size();
                let Some(ranges) = self.binary_ranges(binary, value_bit_size, facts) else {
                    return false;
                };

                if let Some(operands) = binary.operator.backward(result_range, ranges) {
                    self.refine_pair(facts, (binary.lhs, operands.lhs), (binary.rhs, operands.rhs))
                } else {
                    false
                }
            }
            Instruction::Not(original_value) => {
                let original_type = self.dfg.type_of_value(*original_value);
                let Some(range) = result_range.not(original_type.as_ref()) else {
                    return false;
                };
                self.refine(facts, *original_value, range)
            }
            _ => false,
        }
    }

    /// Compute an instruction result range from known facts.
    fn instruction_range(
        &self,
        instruction: &Instruction,
        result: ValueId,
        facts: &Facts,
        allow_field_cast_ranges: bool,
    ) -> Option<ValueRange> {
        let result_type = self.dfg.type_of_value(result);
        if !matches!(result_type.as_ref(), Type::Numeric(_)) {
            return None;
        }
        let value_bit_size = result_type.bit_size();

        match instruction {
            Instruction::Cast(original_value, _) => match result_type.as_ref() {
                Type::Numeric(NumericType::NativeField) => {
                    if !allow_field_cast_ranges {
                        return None;
                    }
                    let original_type = self.dfg.type_of_value(*original_value);
                    facts
                        .range(*original_value)
                        .and_then(|range| range.cast_to_field(original_type.as_ref()))
                        .map(ValueRange::Unsigned)
                }
                Type::Numeric(NumericType::Unsigned { bit_size }) => {
                    let original_type = self.dfg.type_of_value(*original_value);
                    let original_range = facts.range(*original_value)?;
                    original_range
                        .cast_to_unsigned(original_type.as_ref(), *bit_size)
                        .map(ValueRange::Unsigned)
                }
                Type::Numeric(NumericType::Signed { bit_size }) => {
                    let original_type = self.dfg.type_of_value(*original_value);
                    let original_range = facts.range(*original_value)?;
                    original_range
                        .cast_to_signed(original_type.as_ref(), *bit_size)
                        .map(ValueRange::Signed)
                }
                _ => None,
            },
            Instruction::Truncate { value: original_value, bit_size, .. } => self.truncate_range(
                *original_value,
                result_type.as_ref(),
                value_bit_size,
                *bit_size,
                facts,
            ),
            Instruction::Binary(binary) => {
                let ranges = self.binary_ranges(binary, value_bit_size, facts);
                binary.operator.forward(ranges)
            }
            Instruction::Not(original_value) => {
                facts.range(*original_value)?.not(result_type.as_ref())
            }
            _ => None,
        }
    }

    fn initial_range(&self, value: ValueId) -> Option<ValueRange> {
        let value_type = self.dfg.type_of_value(value);
        let Type::Numeric(numeric_type) = value_type.as_ref() else {
            return None;
        };

        match self.dfg[value] {
            Value::NumericConstant {
                constant,
                typ: NumericType::Unsigned { .. } | NumericType::NativeField,
            } => {
                constant.try_into_u128().map(|value| ValueRange::Unsigned(Range::new(value, value)))
            }
            Value::NumericConstant { constant, typ: NumericType::Signed { bit_size } } => {
                signed_constant_value(constant, bit_size)
                    .map(|value| ValueRange::Signed(SignedRange::new(value, value)))
            }
            _ => ValueRange::for_type(numeric_type),
        }
    }

    fn binary_ranges(
        &self,
        binary: &Binary,
        value_bit_size: u32,
        facts: &Facts,
    ) -> Option<BinaryRanges> {
        BinaryRanges::new(
            self.dfg.type_of_value(binary.lhs).unwrap_numeric(),
            value_bit_size,
            facts.range(binary.lhs)?,
            facts.range(binary.rhs)?,
        )
    }

    fn truncate_range(
        &self,
        original_value: ValueId,
        result_type: &Type,
        value_bit_size: u32,
        bit_size: u32,
        facts: &Facts,
    ) -> Option<ValueRange> {
        match result_type {
            Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField) => {
                let max = max_unsigned_value_for_bit_size(value_bit_size.min(bit_size))?;
                let original_range = facts.range(original_value)?;
                original_range
                    .cast_to_unsigned(result_type, value_bit_size.min(bit_size))
                    .map(|range| ValueRange::Unsigned(range.truncate_to(max)))
            }
            Type::Numeric(NumericType::Signed { bit_size: result_bit_size }) => {
                if bit_size >= *result_bit_size {
                    return facts.range(original_value);
                }
                let max = i128::try_from(max_unsigned_value_for_bit_size(bit_size)?).ok()?;
                Some(ValueRange::Signed(SignedRange::new(0, max)))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueRange {
    Unsigned(Range),
    Signed(SignedRange),
}

impl ValueRange {
    fn for_type(typ: &NumericType) -> Option<Self> {
        match typ {
            NumericType::Unsigned { bit_size } => {
                Some(Self::Unsigned(Range::new(0, max_unsigned_value_for_bit_size(*bit_size)?)))
            }
            NumericType::Signed { bit_size } => {
                Some(Self::Signed(SignedRange::for_bit_size(*bit_size)?))
            }
            NumericType::NativeField => None,
        }
    }

    fn max_bits(self, type_bit_size: u32) -> u32 {
        match self {
            Self::Unsigned(range) => range.max_bits(),
            Self::Signed(range) => range.max_bits(type_bit_size),
        }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => lhs.intersect(rhs).map(Self::Unsigned),
            (Self::Signed(lhs), Self::Signed(rhs)) => lhs.intersect(rhs).map(Self::Signed),
            _ => None,
        }
    }

    fn into_unsigned(self) -> Option<Range> {
        match self {
            Self::Unsigned(range) => Some(range),
            Self::Signed(_) => None,
        }
    }

    fn into_signed(self) -> Option<SignedRange> {
        match self {
            Self::Signed(range) => Some(range),
            Self::Unsigned(_) => None,
        }
    }

    fn clamp_to_type(self, typ: &NumericType) -> Option<Self> {
        match (self, typ) {
            (Self::Unsigned(range), NumericType::Unsigned { bit_size }) => {
                let type_max = max_unsigned_value_for_bit_size(*bit_size)?;
                Some(Self::Unsigned(Range::new(range.min.min(type_max), range.max.min(type_max))))
            }
            (Self::Unsigned(range), NumericType::NativeField) => Some(Self::Unsigned(range)),
            (Self::Signed(range), NumericType::Signed { bit_size }) => {
                Some(Self::Signed(range.intersect(SignedRange::for_bit_size(*bit_size)?)?))
            }
            _ => None,
        }
    }

    fn cast_to_field(self, source_type: &Type) -> Option<Range> {
        match self {
            Self::Unsigned(range) => Some(range),
            Self::Signed(range) => {
                let Type::Numeric(NumericType::Signed { bit_size }) = source_type else {
                    return None;
                };
                range.to_unsigned(*bit_size, *bit_size)
            }
        }
    }

    fn cast_to_unsigned(self, source_type: &Type, target_bit_size: u32) -> Option<Range> {
        match self {
            Self::Unsigned(range) => {
                let target_max = max_unsigned_value_for_bit_size(target_bit_size)?;
                Some(range.truncate_to(target_max))
            }
            Self::Signed(range) => {
                let Type::Numeric(NumericType::Signed { bit_size }) = source_type else {
                    return None;
                };
                range.to_unsigned(*bit_size, target_bit_size)
            }
        }
    }

    fn cast_to_signed(self, source_type: &Type, target_bit_size: u32) -> Option<SignedRange> {
        match self {
            Self::Signed(range) => {
                let Type::Numeric(NumericType::Signed { bit_size }) = source_type else {
                    return None;
                };
                range
                    .to_unsigned(*bit_size, *bit_size)
                    .and_then(|range| SignedRange::from_unsigned(range, target_bit_size))
            }
            Self::Unsigned(range) => {
                let Type::Numeric(NumericType::Unsigned { .. } | NumericType::NativeField) =
                    source_type
                else {
                    return None;
                };
                SignedRange::from_unsigned(range, target_bit_size)
            }
        }
    }

    fn not(self, typ: &Type) -> Option<Self> {
        match (self, typ) {
            (Self::Unsigned(range), Type::Numeric(NumericType::Unsigned { bit_size })) => {
                Some(Self::Unsigned(range.not(max_unsigned_value_for_bit_size(*bit_size)?)))
            }
            (Self::Signed(range), Type::Numeric(NumericType::Signed { .. })) => {
                Some(Self::Signed(range.not()))
            }
            _ => None,
        }
    }
}

#[derive(Default, Debug)]
struct Facts {
    ranges: HashMap<ValueId, ValueRange>,
}

impl Facts {
    fn range(&self, value: ValueId) -> Option<ValueRange> {
        self.ranges.get(&value).copied()
    }

    fn set(&mut self, value: ValueId, range: ValueRange) {
        self.ranges.insert(value, range);
    }

    /// Intersect `value`'s current range with `range`.
    ///
    /// Empty refinements are ignored. They can appear when independent conservative facts cannot
    /// overlap, and inventing a replacement singleton would make later inferences unsound.
    fn refine(&mut self, dfg: &DataFlowGraph, value: ValueId, range: ValueRange) -> bool {
        let value_type = dfg.type_of_value(value);
        let Type::Numeric(numeric_type) = value_type.as_ref() else {
            return false;
        };
        let Some(range) = range.clamp_to_type(numeric_type) else {
            return false;
        };

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

/// Inclusive range of possible logical values for a signed SSA value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SignedRange {
    min: i128,
    max: i128,
}

impl SignedRange {
    fn new(min: i128, max: i128) -> Self {
        debug_assert!(min <= max);
        Self { min, max }
    }

    fn for_bit_size(bit_size: u32) -> Option<Self> {
        match bit_size {
            1..=127 => Some(Self::new(-(1i128 << (bit_size - 1)), (1i128 << (bit_size - 1)) - 1)),
            128 => Some(Self::new(i128::MIN, i128::MAX)),
            _ => None,
        }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        if min <= max { Some(Self::new(min, max)) } else { None }
    }

    fn max_bits(self, type_bit_size: u32) -> u32 {
        if self.min < 0 { type_bit_size } else { u128::BITS - (self.max as u128).leading_zeros() }
    }

    fn not(self) -> Self {
        Self::new(!self.max, !self.min)
    }

    fn fixed_shift(self, bit_size: u32) -> Option<u32> {
        if self.min == self.max && self.min >= 0 && self.max < i128::from(bit_size) {
            u32::try_from(self.max).ok()
        } else {
            None
        }
    }

    fn checked_result(
        self,
        rhs: Self,
        type_range: Self,
        operation: impl Fn(i128, i128) -> Option<i128>,
    ) -> Self {
        let candidates = [
            operation(self.min, rhs.min),
            operation(self.min, rhs.max),
            operation(self.max, rhs.min),
            operation(self.max, rhs.max),
        ];
        Self::from_checked_candidates(candidates, type_range)
    }

    fn from_checked_candidates<const N: usize>(
        candidates: [Option<i128>; N],
        type_range: Self,
    ) -> Self {
        let mut min = i128::MAX;
        let mut max = i128::MIN;
        for candidate in candidates {
            let Some(candidate) = candidate else {
                return type_range;
            };
            if candidate < type_range.min || candidate > type_range.max {
                return type_range;
            }
            min = min.min(candidate);
            max = max.max(candidate);
        }
        Self::new(min, max)
    }

    fn contains(self, value: i128) -> bool {
        self.min <= value && value <= self.max
    }

    fn max_abs(self) -> u128 {
        self.min.unsigned_abs().max(self.max.unsigned_abs())
    }

    fn to_unsigned(self, source_bit_size: u32, target_bit_size: u32) -> Option<Range> {
        let target_max = max_unsigned_value_for_bit_size(target_bit_size)?;
        if self.min >= 0 {
            return Some(Range::new(self.min as u128, self.max as u128).truncate_to(target_max));
        }

        if self.max >= 0 || target_bit_size < source_bit_size {
            return Some(Range::new(0, target_max));
        }

        let min = signed_to_twos_complement(self.min, source_bit_size)?;
        let max = signed_to_twos_complement(self.max, source_bit_size)?;
        Some(Range::new(min, max).truncate_to(target_max))
    }

    fn from_unsigned(range: Range, target_bit_size: u32) -> Option<Self> {
        let type_range = Self::for_bit_size(target_bit_size)?;
        let sign_bit = sign_bit(target_bit_size)?;
        let type_max = max_unsigned_value_for_bit_size(target_bit_size)?;

        if range.max <= type_range.max as u128 {
            return Some(Self::new(i128::try_from(range.min).ok()?, range.max as i128));
        }

        if range.min >= sign_bit && range.max <= type_max {
            let min = unsigned_to_signed(range.min, target_bit_size)?;
            let max = unsigned_to_signed(range.max, target_bit_size)?;
            return Some(Self::new(min, max));
        }

        Some(type_range)
    }
}

#[derive(Clone, Copy)]
enum BinaryRanges {
    Unsigned(UnsignedBinaryRanges),
    Signed(SignedBinaryRanges),
}

impl BinaryRanges {
    fn new(typ: NumericType, bit_size: u32, lhs: ValueRange, rhs: ValueRange) -> Option<Self> {
        match typ {
            NumericType::Unsigned { .. } => Some(Self::Unsigned(UnsignedBinaryRanges::new(
                bit_size,
                lhs.into_unsigned()?,
                rhs.into_unsigned()?,
            )?)),
            NumericType::Signed { .. } => Some(Self::Signed(SignedBinaryRanges::new(
                bit_size,
                lhs.into_signed()?,
                rhs.into_signed()?,
            )?)),
            NumericType::NativeField => None,
        }
    }

    fn map(
        self,
        unsigned: impl FnOnce(UnsignedBinaryRanges) -> Range,
        signed: impl FnOnce(SignedBinaryRanges) -> SignedRange,
    ) -> ValueRange {
        match self {
            Self::Unsigned(ranges) => ValueRange::Unsigned(unsigned(ranges)),
            Self::Signed(ranges) => ValueRange::Signed(signed(ranges)),
        }
    }

    fn add(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::add, SignedBinaryRanges::add)
    }

    fn sub(self, unchecked: bool) -> ValueRange {
        self.map(|ranges| ranges.sub(unchecked), SignedBinaryRanges::sub)
    }

    fn mul(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::mul, SignedBinaryRanges::mul)
    }

    fn div(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::div, SignedBinaryRanges::div)
    }

    fn modulo(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::modulo, SignedBinaryRanges::modulo)
    }

    fn bitand(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::bitand, SignedBinaryRanges::bitwise)
    }

    fn bit_or_xor(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::bit_or_xor, SignedBinaryRanges::bitwise)
    }

    fn shl(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::shl, SignedBinaryRanges::shl)
    }

    fn shr(self) -> ValueRange {
        self.map(UnsignedBinaryRanges::shr, SignedBinaryRanges::shr)
    }
}

#[derive(Clone, Copy)]
struct UnsignedBinaryRanges {
    lhs: Range,
    rhs: Range,
    bit_size: u32,
    type_max: u128,
}

impl UnsignedBinaryRanges {
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
            let overflow_possible = self.lhs.max > (self.type_max >> shift);
            let max = if overflow_possible { self.type_max } else { self.lhs.max << shift };
            let min = if overflow_possible { 0 } else { self.lhs.min << shift };
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

#[derive(Clone, Copy)]
struct SignedBinaryRanges {
    lhs: SignedRange,
    rhs: SignedRange,
    bit_size: u32,
    type_range: SignedRange,
}

impl SignedBinaryRanges {
    fn new(bit_size: u32, lhs: SignedRange, rhs: SignedRange) -> Option<Self> {
        Some(Self { lhs, rhs, bit_size, type_range: SignedRange::for_bit_size(bit_size)? })
    }

    fn add(self) -> SignedRange {
        self.lhs.checked_result(self.rhs, self.type_range, i128::checked_add)
    }

    fn sub(self) -> SignedRange {
        let candidates =
            [self.lhs.min.checked_sub(self.rhs.max), self.lhs.max.checked_sub(self.rhs.min)];
        SignedRange::from_checked_candidates(candidates, self.type_range)
    }

    fn mul(self) -> SignedRange {
        self.lhs.checked_result(self.rhs, self.type_range, i128::checked_mul)
    }

    fn div(self) -> SignedRange {
        if self.div_mod_can_fail() {
            return self.type_range;
        }

        self.lhs.checked_result(self.rhs, self.type_range, i128::checked_div)
    }

    fn modulo(self) -> SignedRange {
        if self.div_mod_can_fail() {
            return self.type_range;
        }

        let max_abs_rhs = self.rhs.max_abs().saturating_sub(1);
        let max_magnitude = max_abs_rhs.min(i128::MAX as u128) as i128;

        if self.lhs.max < 0 {
            let lhs_magnitude = self.lhs.max_abs().min(i128::MAX as u128) as i128;
            SignedRange::new(-max_magnitude.min(lhs_magnitude), 0)
        } else if self.lhs.min >= 0 {
            SignedRange::new(0, max_magnitude.min(self.lhs.max))
        } else {
            SignedRange::new(-max_magnitude, max_magnitude)
        }
    }

    fn bitwise(self) -> SignedRange {
        self.type_range
    }

    fn div_mod_can_fail(self) -> bool {
        self.rhs.contains(0) || (self.lhs.contains(self.type_range.min) && self.rhs.contains(-1))
    }

    fn shl(self) -> SignedRange {
        let Some(shift) = self.rhs.fixed_shift(self.bit_size) else {
            return self.type_range;
        };

        self.lhs.checked_result(
            SignedRange::new(shift.into(), shift.into()),
            self.type_range,
            |lhs, rhs| lhs.checked_shl(u32::try_from(rhs).ok()?),
        )
    }

    fn shr(self) -> SignedRange {
        let Some(shift) = self.rhs.fixed_shift(self.bit_size) else {
            return self.type_range;
        };

        SignedRange::new(self.lhs.min >> shift, self.lhs.max >> shift)
    }
}

struct UnsignedBinaryBack {
    result: Range,
    ranges: UnsignedBinaryRanges,
}

impl UnsignedBinaryBack {
    fn add(&self, unchecked: bool) -> Option<OperandRanges<Range>> {
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

    fn sub(&self, unchecked: bool) -> Option<OperandRanges<Range>> {
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

    fn mul(&self, unchecked: bool) -> Option<OperandRanges<Range>> {
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

    fn div(&self) -> Option<OperandRanges<Range>> {
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

    fn bitor(&self) -> Option<OperandRanges<Range>> {
        Some(
            self.operands()
                .tighten_lhs(Range::new(0, self.result.max))
                .tighten_rhs(Range::new(0, self.result.max)),
        )
    }

    fn shl(&self) -> Option<OperandRanges<Range>> {
        if self.ranges.rhs.min != self.ranges.rhs.max || self.ranges.rhs.max >= 128 {
            return None;
        }

        let shift = self.ranges.rhs.max as u32;
        if self.ranges.lhs.max > (self.ranges.type_max >> shift) {
            return None;
        }

        Some(self.operands().tighten_lhs(Range::new(0, self.result.max >> shift)))
    }

    fn operands(&self) -> OperandRanges<Range> {
        OperandRanges { lhs: self.ranges.lhs, rhs: self.ranges.rhs }
    }

    fn result_may_wrap(&self, operation: impl FnOnce(u128, u128) -> Option<u128>) -> bool {
        operation(self.ranges.lhs.max, self.ranges.rhs.max)
            .is_none_or(|result| result > self.ranges.type_max)
    }
}

trait IntersectRange: Copy {
    fn intersect(self, other: Self) -> Option<Self>;
}

impl IntersectRange for Range {
    fn intersect(self, other: Self) -> Option<Self> {
        Range::intersect(self, other)
    }
}

impl IntersectRange for SignedRange {
    fn intersect(self, other: Self) -> Option<Self> {
        SignedRange::intersect(self, other)
    }
}

struct OperandRanges<T> {
    lhs: T,
    rhs: T,
}

impl<T> OperandRanges<T> {
    fn map<U>(self, f: impl Fn(T) -> U) -> OperandRanges<U> {
        OperandRanges { lhs: f(self.lhs), rhs: f(self.rhs) }
    }
}

impl<T: IntersectRange> OperandRanges<T> {
    fn tighten_lhs(mut self, range: T) -> Self {
        if let Some(range) = self.lhs.intersect(range) {
            self.lhs = range;
        }
        self
    }

    fn tighten_rhs(mut self, range: T) -> Self {
        if let Some(range) = self.rhs.intersect(range) {
            self.rhs = range;
        }
        self
    }
}

impl OperandRanges<Range> {
    // Empty intersections are ignored to match `Facts::refine`.
    fn tighten_lhs_bounds(self, min: u128, max: u128) -> Self {
        if min <= max { self.tighten_lhs(Range::new(min, max)) } else { self }
    }

    fn tighten_rhs_bounds(self, min: u128, max: u128) -> Self {
        if min <= max { self.tighten_rhs(Range::new(min, max)) } else { self }
    }
}

struct SignedBinaryBack {
    result: SignedRange,
    ranges: SignedBinaryRanges,
}

impl SignedBinaryBack {
    fn add(&self) -> Option<OperandRanges<SignedRange>> {
        let lhs = self.bounds(
            self.result.min.checked_sub(self.ranges.rhs.max),
            self.result.max.checked_sub(self.ranges.rhs.min),
        )?;
        let rhs = self.bounds(
            self.result.min.checked_sub(self.ranges.lhs.max),
            self.result.max.checked_sub(self.ranges.lhs.min),
        )?;
        Some(self.operands().tighten_lhs(lhs).tighten_rhs(rhs))
    }

    fn sub(&self) -> Option<OperandRanges<SignedRange>> {
        let lhs = self.bounds(
            self.result.min.checked_add(self.ranges.rhs.min),
            self.result.max.checked_add(self.ranges.rhs.max),
        )?;
        let rhs = self.bounds(
            self.ranges.lhs.min.checked_sub(self.result.max),
            self.ranges.lhs.max.checked_sub(self.result.min),
        )?;
        Some(self.operands().tighten_lhs(lhs).tighten_rhs(rhs))
    }

    fn operands(&self) -> OperandRanges<SignedRange> {
        OperandRanges { lhs: self.ranges.lhs, rhs: self.ranges.rhs }
    }

    fn bounds(&self, min: Option<i128>, max: Option<i128>) -> Option<SignedRange> {
        let min = min?.max(self.ranges.type_range.min);
        let max = max?.min(self.ranges.type_range.max);
        (min <= max).then(|| SignedRange::new(min, max))
    }
}

impl BinaryOp {
    fn forward(self, ranges: Option<BinaryRanges>) -> Option<ValueRange> {
        match self {
            BinaryOp::Eq | BinaryOp::Lt => Some(ValueRange::Unsigned(Range::new(0, 1))),
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

    fn backward(
        self,
        result: ValueRange,
        ranges: BinaryRanges,
    ) -> Option<OperandRanges<ValueRange>> {
        match (result, ranges) {
            (ValueRange::Unsigned(result), BinaryRanges::Unsigned(ranges)) => {
                let back = UnsignedBinaryBack { result, ranges };
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
                operands.map(|operands| operands.map(ValueRange::Unsigned))
            }
            (ValueRange::Signed(result), BinaryRanges::Signed(ranges)) => {
                let back = SignedBinaryBack { result, ranges };
                let operands = match self {
                    BinaryOp::Add { unchecked: false } => back.add(),
                    BinaryOp::Sub { unchecked: false } => back.sub(),
                    _ => None,
                };
                operands.map(|operands| operands.map(ValueRange::Signed))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U8_BITS: u32 = 8;

    fn u8_ranges(lhs: Range, rhs: Range) -> UnsignedBinaryRanges {
        UnsignedBinaryRanges::new(U8_BITS, lhs, rhs).unwrap()
    }

    fn u8_back(
        lhs_range: Range,
        rhs_range: Range,
        result: Range,
        operator: BinaryOp,
    ) -> (Range, Range) {
        let ranges = u8_ranges(lhs_range, rhs_range);
        let operands = operator
            .backward(ValueRange::Unsigned(result), BinaryRanges::Unsigned(ranges))
            .expect("u8 back-propagation should return operand ranges");

        (operands.lhs.into_unsigned().unwrap(), operands.rhs.into_unsigned().unwrap())
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
    fn signed_range_max_bits_uses_full_width_for_negative_values() {
        assert_eq!(SignedRange::new(0, 7).max_bits(8), 3);
        assert_eq!(SignedRange::new(-1, 7).max_bits(8), 8);
    }

    #[test]
    fn signed_range_add_uses_precise_bounds_or_full_range_on_overflow() {
        let ranges =
            SignedBinaryRanges::new(8, SignedRange::new(-10, 20), SignedRange::new(2, 3)).unwrap();
        assert_eq!(ranges.add(), SignedRange::new(-8, 23));

        let ranges =
            SignedBinaryRanges::new(8, SignedRange::new(120, 127), SignedRange::new(1, 2)).unwrap();
        assert_eq!(ranges.add(), SignedRange::new(-128, 127));
    }

    #[test]
    fn signed_range_mul_checks_all_interval_corners() {
        let ranges =
            SignedBinaryRanges::new(8, SignedRange::new(-4, 6), SignedRange::new(-3, 5)).unwrap();

        assert_eq!(ranges.mul(), SignedRange::new(-20, 30));
    }

    #[test]
    fn signed_range_modulo_tracks_result_sign() {
        let ranges =
            SignedBinaryRanges::new(8, SignedRange::new(-10, 20), SignedRange::new(3, 5)).unwrap();

        assert_eq!(ranges.modulo(), SignedRange::new(-4, 4));
    }

    #[test]
    fn signed_casts_handle_contiguous_twos_complement_ranges() {
        let negative = SignedRange::new(-16, -1);
        assert_eq!(negative.to_unsigned(8, 16), Some(Range::new(240, 255)));
        assert_eq!(
            SignedRange::from_unsigned(Range::new(240, 255), 8),
            Some(SignedRange::new(-16, -1))
        );
        assert_eq!(
            ValueRange::Signed(negative).cast_to_signed(&Type::signed(8), 16),
            Some(SignedRange::new(240, 255))
        );
    }

    #[test]
    fn instruction_ranges_use_initialized_type_facts_for_casts() {
        let mut dfg = DataFlowGraph::default();
        let block = dfg.make_block();
        let original = dfg.add_block_parameter(block, Type::signed(16));
        let result = dfg.add_block_parameter(block, Type::unsigned(8));
        let mut facts = Facts::default();
        facts.set(original, ValueRange::Signed(SignedRange::for_bit_size(16).unwrap()));
        let analysis = Analysis::new(&dfg);

        let range = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::unsigned(8)),
            result,
            &facts,
            true,
        );

        assert_eq!(range, Some(ValueRange::Unsigned(Range::new(0, 255))));
    }

    #[test]
    fn field_casts_propagate_known_facts() {
        let mut dfg = DataFlowGraph::default();
        let block = dfg.make_block();
        let original = dfg.add_block_parameter(block, Type::unsigned(8));
        let result = dfg.add_block_parameter(block, Type::field());
        let mut facts = Facts::default();
        facts.set(original, ValueRange::Unsigned(Range::new(3, 7)));
        let analysis = Analysis::new(&dfg);

        let range_without_global_constraints = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::NativeField),
            result,
            &facts,
            false,
        );
        let range_with_global_constraints = analysis.instruction_range(
            &Instruction::Cast(original, NumericType::NativeField),
            result,
            &facts,
            true,
        );

        assert_eq!(range_without_global_constraints, None);
        assert_eq!(range_with_global_constraints, Some(ValueRange::Unsigned(Range::new(3, 7))));
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
    fn binary_ranges_shl_falls_back_when_shift_may_wrap() {
        let ranges = u8_ranges(Range::new(128, 255), Range::new(1, 1));

        assert_eq!(ranges.shl(), Range::new(0, 255));
    }

    #[test]
    fn binary_back_add_refines_both_operands() {
        let (lhs, rhs) = u8_back(
            Range::new(0, 255),
            Range::new(0, 255),
            Range::new(0, 15),
            BinaryOp::Add { unchecked: false },
        );

        assert_eq!(lhs, Range::new(0, 15));
        assert_eq!(rhs, Range::new(0, 15));
    }

    #[test]
    fn binary_back_sub_refines_checked_operands() {
        let (lhs, rhs) = u8_back(
            Range::new(0, 255),
            Range::new(0, 255),
            Range::new(10, 20),
            BinaryOp::Sub { unchecked: false },
        );

        assert_eq!(lhs, Range::new(10, 255));
        assert_eq!(rhs, Range::new(0, 245));
    }

    #[test]
    fn binary_back_mul_refines_positive_operands() {
        let (lhs, rhs) = u8_back(
            Range::new(1, 255),
            Range::new(1, 255),
            Range::new(4, 20),
            BinaryOp::Mul { unchecked: false },
        );

        assert_eq!(lhs, Range::new(1, 20));
        assert_eq!(rhs, Range::new(1, 20));
    }

    #[test]
    fn binary_back_div_refines_lhs_from_nonzero_rhs() {
        let (lhs, rhs) =
            u8_back(Range::new(0, 255), Range::new(2, 10), Range::new(3, 4), BinaryOp::Div);

        assert_eq!(lhs, Range::new(6, 49));
        assert_eq!(rhs, Range::new(2, 10));
    }

    #[test]
    fn binary_back_shl_refines_lhs_for_fixed_shift() {
        let (lhs, rhs) =
            u8_back(Range::new(0, 63), Range::new(2, 2), Range::new(0, 31), BinaryOp::Shl);

        assert_eq!(lhs, Range::new(0, 7));
        assert_eq!(rhs, Range::new(2, 2));
    }

    #[test]
    fn binary_back_shl_does_not_refine_wrapping_shift() {
        let ranges = u8_ranges(Range::new(128, 255), Range::new(1, 1));
        let operands = BinaryOp::Shl
            .backward(ValueRange::Unsigned(Range::new(0, 31)), BinaryRanges::Unsigned(ranges))
            .and_then(|operands| operands.lhs.into_unsigned());

        assert_eq!(operands, None);
    }
}
