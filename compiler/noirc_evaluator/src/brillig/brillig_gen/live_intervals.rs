#![allow(dead_code, unused_imports)]
//! Liveness intervals for Brillig code generation.
//!
//! This module provides a global per-value interval representation complementing
//! the per-block set-based view in [VariableLiveness]. Each SSA value is assigned
//! a conservative `[def, last_use]` range over a total program-point ordering.
//!
//! Single-interval representation: we use one contiguous interval per value, which
//! over-approximates non-contiguous live ranges. This is sound for interference
//! checks (two values whose intervals overlap might interfere) but may report
//! false positives. A future extension can use interval sets for precision.
//!
//! Current uses:
//! - Register pressure analysis
//! - Foundation for coalescing validation
//! - Foundation for a future linear-scan register allocator
//!
//! # References
//!
//! The interval-building algorithm is based on the BUILDINTERVALS procedure from:
//!
//! - Wimmer & Franz, "Linear Scan Register Allocation on SSA Form", CGO 2010
//!   (DOI: `10.1145/1772954.1772979`), Section 4.1.
//!
//! Rather than re-deriving loop liveness via Wimmer's loop-header clause, we follow
//! the practical adaptation described at <https://bernsteinbear.com/blog/linear-scan/>
//! and reuse the pre-existing loop-aware liveness sets already computed by
//! [VariableLiveness]. This simplifies the implementation while preserving the
//! same interval semantics.
//!
//! Long-term, these intervals are intended to feed a full linear-scan register
//! allocator (replacing the current LRU spilling strategy), which is why the
//! Wimmer & Franz paper is the primary reference.

use crate::ssa::ir::{
    basic_block::BasicBlockId, function::Function, instruction::InstructionId, value::ValueId,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{constant_allocation::ConstantAllocation, variable_liveness::VariableLiveness};

/// Monotonic index assigned to block entries, instructions, and terminators in RPO.
///
/// Program points form a total order over the function. A lower value means
/// "earlier in execution" (in RPO traversal order). This is used to define
/// intervals and check overlap.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProgramPoint(u32);

/// Conservative single-interval approximation of a value's liveness.
///
/// Over-approximates for non-contiguous live ranges (sound for interference).
/// `def` is the program point where the value is defined, and `last_use` is the
/// last program point where it is consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LiveInterval {
    pub(crate) def: ProgramPoint,
    pub(crate) last_use: ProgramPoint,
}

impl LiveInterval {
    fn new(def: ProgramPoint, last_use: ProgramPoint) -> Self {
        assert!(def <= last_use);
        Self { def, last_use }
    }

    /// Check whether two intervals overlap (conservative interference).
    #[cfg(test)]
    pub(crate) fn interferes(&self, other: &LiveInterval) -> bool {
        self.def <= other.last_use && other.def <= self.last_use
    }

    /// Check whether a program point falls within this interval.
    #[cfg(test)]
    pub(crate) fn contains(&self, point: ProgramPoint) -> bool {
        self.def <= point && point <= self.last_use
    }
}

/// Global liveness intervals for all values in a function.
///
/// Built from [VariableLiveness] (which provides loop-aware `live_in`/`live_out`)
/// and [ConstantAllocation] (which provides constant allocation points).
#[derive(Default)]
pub(crate) struct LiveIntervals {
    /// Map from block to the program point at its entry.
    block_entry_points: HashMap<BasicBlockId, ProgramPoint>,
    /// Map from instruction to its program point.
    instruction_points: HashMap<InstructionId, ProgramPoint>,
    /// Map from block to its terminator's program point.
    terminator_points: HashMap<BasicBlockId, ProgramPoint>,
    /// Per-value liveness interval.
    intervals: HashMap<ValueId, LiveInterval>,
    /// Maximum program point assigned (for iteration bounds).
    max_point: ProgramPoint,
}

#[cfg(test)]
impl LiveIntervals {
    /// Build liveness intervals for a function.
    ///
    /// See the module-level docs for algorithm references and [Self::build_intervals]
    /// for the annotated pseudocode mapping.
    pub(crate) fn from_function(
        func: &Function,
        liveness: &VariableLiveness,
        constants: &ConstantAllocation,
        post_order: &[BasicBlockId],
    ) -> Self {
        let mut result = Self {
            block_entry_points: HashMap::default(),
            instruction_points: HashMap::default(),
            terminator_points: HashMap::default(),
            intervals: HashMap::default(),
            max_point: ProgramPoint(0),
        };

        // Step 0: Assign program points in reverse post-order (RPO).
        result.assign_program_points(func, post_order);

        // Step 1: Build intervals by processing blocks in post-order.
        result.build_intervals(func, liveness, post_order);

        // Step 2: Post-process for Noir's idom allocation of block params.
        result.adjust_block_param_defs(liveness, post_order);

        // Step 3: Handle constants allocated at specific block entries.
        result.adjust_constant_defs(constants, post_order);

        result
    }

    /// Assign monotonically increasing program points to block entries,
    /// instructions, and terminators in RPO.
    fn assign_program_points(&mut self, func: &Function, post_order: &[BasicBlockId]) {
        let mut index: u32 = 0;

        for &block_id in post_order.iter().rev() {
            // Block entry point.
            self.block_entry_points.insert(block_id, ProgramPoint(index));
            index += 1;

            // Instructions.
            let block = &func.dfg[block_id];
            for &inst_id in block.instructions() {
                self.instruction_points.insert(inst_id, ProgramPoint(index));
                index += 1;
            }

            // Terminator.
            // Every block in valid SSA must have one.
            block.unwrap_terminator();
            self.terminator_points.insert(block_id, ProgramPoint(index));
            index += 1;
        }

        self.max_point = ProgramPoint(index.saturating_sub(1));
    }

    /// Build intervals by processing blocks in post-order (reverse of RPO).
    ///
    /// Implements BUILDINTERVALS from Wimmer & Franz 2010, Section 4.1.
    /// The pseudocode below shows the original algorithm on the left and our
    /// mapping on the right:
    ///
    /// ```text
    /// BUILDINTERVALS
    ///
    /// for each block b in reverse order do          <- post_order iter (== reverse RPO)
    ///   live = union of successor.liveIn for each   <- liveness.get_live_out() (equivalent:
    ///          successor of b                         live_out = ∪ live_in(s) for successors s)
    ///   for each phi function phi of successors     <- terminator.for_each_value() — adds jmp args
    ///          of b do                                (phi inputs), jmpif conditions, return values
    ///     live.add(phi.inputOf(b))
    ///   for each opd in live do                     <- the `for &v in &live` loop
    ///     intervals[opd].addRange(b.from, b.to)
    ///   for each operation op of b in reverse do    <- block.instructions().iter().rev()
    ///     for each output operand opd of op do      <- func.dfg.instruction_results()
    ///       intervals[opd].setFrom(op.id)
    ///       live.remove(opd)                        <- SKIPPED: `live` is not stored. It is only used to seed intervals
    ///     for each input operand opd of op do       <- instruction.for_each_value()
    ///       intervals[opd].addRange(b.from, op.id)
    ///       live.add(opd)                           <- SKIPPED: `live` is not stored. It is only used to seed intervals
    ///   for each phi function phi of b do           <- SKIPPED: `live` is not stored. It is only used to seed intervals
    ///     live.remove(phi.output)
    ///   if b is loop header then                    <- SKIPPED: VariableLiveness already propagates
    ///     loopEnd = last block of loop at b           loop liveness through back-edges, so live_out
    ///     for each opd in live do                     already includes loop-carried values.
    ///       intervals[opd].addRange(b.from,           See <https://bernsteinbear.com/blog/linear-scan/>
    ///                               loopEnd.to)
    ///   b.liveIn = live                             <- NOT STORED: we don't need per-block liveIn
    ///                                                 since VariableLiveness already provides it.
    /// ```
    ///
    /// Key differences from the paper:
    /// - `live_out` vs `union of successor.liveIn`: These are equivalent —
    ///   [VariableLiveness::get_live_out()] is literally `∪ live_in(s) for each successor s`.
    /// - Phi inputs -> terminator operands: Wimmer adds phi inputs explicitly;
    ///   we add all terminator operands (jmp args are the phi inputs in Noir's
    ///   block-parameter SSA form).
    /// - Loop header clause skipped: [VariableLiveness] already handles loop
    ///   liveness propagation, so `live_out` includes loop-carried values.
    /// - `b.liveIn` not stored: Wimmer builds liveIn bottom-up; we already
    ///   have it from [VariableLiveness].
    fn build_intervals(
        &mut self,
        func: &Function,
        liveness: &VariableLiveness,
        post_order: &[BasicBlockId],
    ) {
        // Process blocks in post-order so that successors are
        // processed before predecessors, matching the Wimmer algorithm.
        for &block_id in post_order {
            let block = &func.dfg[block_id];
            let block_entry = self.block_entry_points[&block_id];
            let block_end = self.terminator_points[&block_id];

            // Start with live_out: values alive after the block.
            let mut live: HashSet<ValueId> = liveness.get_live_out(&block_id);

            // Add ALL terminator operands to the live set.
            // Jmp arguments (phi inputs), jmpif conditions, return values --
            // these must be alive at the terminator but may not be in live_out.
            let terminator = block.unwrap_terminator();
            terminator.for_each_value(|v| {
                if super::variable_liveness::is_variable(v, &func.dfg) {
                    live.insert(v);
                }
            });

            // All live values get a range covering the entire block.
            for &v in &live {
                self.add_range(v, block_entry, block_end);
            }

            // Walk instructions in reverse order.
            for &inst_id in block.instructions().iter().rev() {
                let inst_point = self.instruction_points[&inst_id];

                // For each output: set def to this instruction.
                let results = func.dfg.instruction_results(inst_id);
                for &result in results {
                    self.set_from(result, inst_point);
                }

                // For each input: add range from block entry to this instruction.
                let instruction = &func.dfg[inst_id];
                instruction.for_each_value(|v| {
                    if super::variable_liveness::is_variable(v, &func.dfg) {
                        self.add_range(v, block_entry, inst_point);
                    }
                });
            }
        }
    }

    /// Extend each block param's def backward to its idom's entry point.
    ///
    /// In Brillig, block params are allocated at their immediate dominator
    /// (the last point before the branches diverge). This matches the current
    /// codegen behavior where param registers are reserved at the idom.
    fn adjust_block_param_defs(
        &mut self,
        liveness: &VariableLiveness,
        post_order: &[BasicBlockId],
    ) {
        for &block_id in post_order {
            let entry_point = self.block_entry_points[&block_id];
            for param in liveness.defined_block_params(&block_id) {
                if let Some(interval) = self.intervals.get_mut(&param) {
                    interval.def = std::cmp::min(interval.def, entry_point);
                }
            }
        }
    }

    /// Extend constant defs to their allocation block's entry point.
    ///
    /// Constants from `ConstantAllocation::allocated_in_block` are materialized
    /// at specific block entries. We extend their def to match.
    fn adjust_constant_defs(
        &mut self,
        constants: &ConstantAllocation,
        post_order: &[BasicBlockId],
    ) {
        for &block_id in post_order {
            let entry_point = self.block_entry_points[&block_id];
            for constant_id in constants.allocated_in_block(block_id) {
                if let Some(interval) = self.intervals.get_mut(&constant_id) {
                    interval.def = std::cmp::min(interval.def, entry_point);
                }
            }
        }
    }

    /// Add a range `[from, to]` for a value. If the value already has an interval,
    /// extend it to cover the union.
    fn add_range(&mut self, value: ValueId, from: ProgramPoint, to: ProgramPoint) {
        self.intervals
            .entry(value)
            .and_modify(|iv| {
                iv.def = std::cmp::min(iv.def, from);
                iv.last_use = std::cmp::max(iv.last_use, to);
            })
            .or_insert(LiveInterval::new(from, to));
    }

    /// Set the def point for a value, narrowing it to the actual definition.
    fn set_from(&mut self, value: ValueId, point: ProgramPoint) {
        self.intervals
            .entry(value)
            .and_modify(|iv| iv.def = point)
            .or_insert(LiveInterval::new(point, point));
    }

    /// Look up the interval for a value.
    #[cfg(test)]
    pub(crate) fn get(&self, value: ValueId) -> Option<&LiveInterval> {
        self.intervals.get(&value)
    }

    /// Check whether two values' intervals overlap.
    #[cfg(test)]
    pub(crate) fn interferes(&self, a: ValueId, b: ValueId) -> bool {
        match (self.intervals.get(&a), self.intervals.get(&b)) {
            (Some(ia), Some(ib)) => ia.interferes(ib),
            _ => false,
        }
    }

    /// Get the program point at a block's entry.
    #[cfg(test)]
    pub(crate) fn block_entry_point(&self, block: BasicBlockId) -> Option<ProgramPoint> {
        self.block_entry_points.get(&block).copied()
    }

    /// Get the program point for an instruction.
    #[cfg(test)]
    pub(crate) fn instruction_point(&self, inst: InstructionId) -> Option<ProgramPoint> {
        self.instruction_points.get(&inst).copied()
    }

    /// Get the program point for a block's terminator.
    #[cfg(test)]
    pub(crate) fn terminator_point(&self, block: BasicBlockId) -> Option<ProgramPoint> {
        self.terminator_points.get(&block).copied()
    }

    /// Count how many value intervals are live at a given block's entry point.
    #[cfg(test)]
    pub(crate) fn register_pressure_at_block(&self, block: BasicBlockId) -> usize {
        let Some(&entry) = self.block_entry_points.get(&block) else {
            return 0;
        };
        self.intervals.values().filter(|iv| iv.contains(entry)).count()
    }

    /// Return the maximum number of simultaneously live values across all program points.
    ///
    /// This is an approximation. We check at every assigned program point how many
    /// intervals are alive. For large functions a sweep-line algorithm would be
    /// more efficient, but this is adequate for diagnostic use.
    #[cfg(test)]
    pub(crate) fn max_register_pressure(&self) -> usize {
        if self.intervals.is_empty() {
            return 0;
        }

        // Collect all interval endpoints and sweep.
        let mut events: Vec<(u32, i32)> = Vec::new();
        for interval in self.intervals.values() {
            events.push((interval.def.0, 1)); // interval starts
            events.push((interval.last_use.0 + 1, -1)); // interval ends (exclusive)
        }
        events.sort();

        let mut max_pressure = 0usize;
        let mut current = 0i32;
        for (_, delta) in events {
            current += delta;
            max_pressure = max_pressure.max(current as usize);
        }
        max_pressure
    }
}

#[cfg(test)]
mod tests {
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::basic_block::BasicBlockId;
    use crate::ssa::ir::function::Function;
    use crate::ssa::ir::instruction::InstructionId;
    use crate::ssa::ir::post_order::PostOrder;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    use super::LiveIntervals;

    pub(super) fn block_ids<const N: usize>() -> [BasicBlockId; N] {
        std::array::from_fn(|i| BasicBlockId::new(i as u32))
    }

    /// Build LiveIntervals from SSA source string.
    pub(super) fn build_intervals(src: &str) -> (LiveIntervals, Ssa) {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let post_order = PostOrder::with_function(func).into_vec();
        let intervals = LiveIntervals::from_function(func, &liveness, &constants, &post_order);
        (intervals, ssa)
    }

    /// Get the n-th instruction ID in a block.
    fn inst_id(func: &Function, block: BasicBlockId, n: usize) -> InstructionId {
        func.dfg[block].instructions()[n]
    }

    /// Get the single result of the n-th instruction in a block.
    fn inst_result(func: &Function, block: BasicBlockId, n: usize) -> ValueId {
        func.dfg.instruction_results(inst_id(func, block, n))[0]
    }

    #[test]
    fn linear_function() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            v3 = add v2, Field 3
            return v3
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0] = block_ids();
        assert_eq!(rpo, vec![b0]);

        let v0 = func.dfg[b0].parameters()[0];
        let v1 = inst_result(func, b0, 0);
        let v2 = inst_result(func, b0, 1);
        let v3 = inst_result(func, b0, 2);

        let inst0 = inst_id(func, b0, 0);
        let inst1 = inst_id(func, b0, 1);
        let inst2 = inst_id(func, b0, 2);

        let b0_entry = intervals.block_entry_point(b0).unwrap();
        let inst0_pt = intervals.instruction_point(inst0).unwrap();
        let inst1_pt = intervals.instruction_point(inst1).unwrap();
        let inst2_pt = intervals.instruction_point(inst2).unwrap();
        let term_pt = intervals.terminator_point(b0).unwrap();

        let iv0 = intervals.get(v0).expect("v0 should have an interval");
        let iv1 = intervals.get(v1).expect("v1 should have an interval");
        let iv2 = intervals.get(v2).expect("v2 should have an interval");
        let iv3 = intervals.get(v3).expect("v3 should have an interval");

        // Exact interval assertions.
        assert_eq!(iv0.def, b0_entry, "v0 def");
        assert_eq!(iv0.last_use, inst0_pt, "v0 last_use");
        assert_eq!(iv1.def, inst0_pt, "v1 def");
        assert_eq!(iv1.last_use, inst1_pt, "v1 last_use");
        assert_eq!(iv2.def, inst1_pt, "v2 def");
        assert_eq!(iv2.last_use, inst2_pt, "v2 last_use");
        assert_eq!(iv3.def, inst2_pt, "v3 def");
        assert_eq!(iv3.last_use, term_pt, "v3 last_use");

        // Interference: adjacent values share a boundary and interfere,
        // non-adjacent values don't.
        assert!(intervals.interferes(v0, v1), "v0 and v1 should interfere (adjacent)");
        assert!(!intervals.interferes(v0, v2), "v0 and v2 should not interfere");
        assert!(!intervals.interferes(v0, v3), "v0 and v3 should not interfere");
        assert!(!intervals.interferes(v1, v3), "v1 and v3 should not interfere");
    }

    #[test]
    fn diamond_cfg() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            jmpif v0 then: b1(), else: b2()
          b1():
            v2 = add v1, Field 1
            jmp b3(v2)
          b2():
            v3 = mul v1, Field 2
            jmp b3(v3)
          b3(v4: Field):
            return v4
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3] = block_ids();
        assert_eq!(rpo, vec![b0, b2, b1, b3]);

        let v0 = func.dfg[b0].parameters()[0];
        let v1 = func.dfg[b0].parameters()[1];
        let v2 = inst_result(func, b1, 0);
        let v3 = inst_result(func, b2, 0);
        let v4 = func.dfg[b3].parameters()[0];

        let b1_inst = inst_id(func, b1, 0);
        let b2_inst = inst_id(func, b2, 0);

        let b0_entry = intervals.block_entry_point(b0).unwrap();
        let b0_term = intervals.terminator_point(b0).unwrap();
        let b1_inst_pt = intervals.instruction_point(b1_inst).unwrap();
        let b1_term = intervals.terminator_point(b1).unwrap();
        let b2_inst_pt = intervals.instruction_point(b2_inst).unwrap();
        let b2_term = intervals.terminator_point(b2).unwrap();
        let b3_term = intervals.terminator_point(b3).unwrap();

        let iv0 = intervals.get(v0).expect("v0 should have an interval");
        let iv1 = intervals.get(v1).expect("v1 should have an interval");
        let iv2 = intervals.get(v2).expect("v2 should have an interval");
        let iv3 = intervals.get(v3).expect("v3 should have an interval");
        let iv4 = intervals.get(v4).expect("v4 should have an interval");

        // Exact interval assertions.
        assert_eq!(iv0.def, b0_entry, "v0 def");
        assert_eq!(iv0.last_use, b0_term, "v0 last_use");
        assert_eq!(iv1.def, b0_entry, "v1 def");
        // v1 is used in both branches; RPO ordering determines which point is later.
        assert_eq!(iv1.last_use, b1_inst_pt, "v1 last_use");
        assert_eq!(iv2.def, b1_inst_pt, "v2 def");
        assert_eq!(iv2.last_use, b1_term, "v2 last_use");
        assert_eq!(iv3.def, b2_inst_pt, "v3 def");
        assert_eq!(iv3.last_use, b2_term, "v3 last_use");
        assert_eq!(iv4.def, b0_entry, "v4 def (idom adjusted to b0)");
        assert_eq!(iv4.last_use, b3_term, "v4 last_use");

        // Interference checks.
        assert!(!intervals.interferes(v2, v3), "v2 and v3 in different branches");
        assert!(intervals.interferes(v0, v1), "v0 and v1 both span from entry(b0)");
        assert!(intervals.interferes(v1, v4), "v1 and v4 overlap (v4 from idom)");
        assert!(intervals.interferes(v0, v1), "v0 and v4 overlap");
    }

    #[test]
    fn simple_loop() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, v0
            jmpif v2 then: b2(), else: b3()
          b2():
            v3 = add v1, u32 1
            jmp b1(v3)
          b3():
            return v1
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3] = block_ids();
        assert_eq!(rpo, vec![b0, b1, b3, b2]);

        let v0 = func.dfg[b0].parameters()[0];
        let v1 = func.dfg[b1].parameters()[0];
        let v2 = inst_result(func, b1, 0);
        let v3 = inst_result(func, b2, 0);

        let b1_inst = inst_id(func, b1, 0);
        let b2_inst = inst_id(func, b2, 0);

        let b0_entry = intervals.block_entry_point(b0).unwrap();
        let b1_inst_pt = intervals.instruction_point(b1_inst).unwrap();
        let b1_term = intervals.terminator_point(b1).unwrap();
        let b2_inst_pt = intervals.instruction_point(b2_inst).unwrap();
        let b2_term = intervals.terminator_point(b2).unwrap();

        let iv0 = intervals.get(v0).expect("v0 should have an interval");
        let iv1 = intervals.get(v1).expect("v1 should have an interval");
        let iv2 = intervals.get(v2).expect("v2 should have an interval");
        let iv3 = intervals.get(v3).expect("v3 should have an interval");

        // Exact interval assertions.
        // v0 is live through b2 (back-edge)
        assert_eq!(iv0.def, b0_entry, "v0 def");
        assert_eq!(iv0.last_use, b2_term, "v0 last_use");

        assert_eq!(iv1.def, b0_entry, "v1 def (idom adjusted to b0)");
        // v1 (loop header param) is live through b2 (back-edge) and b3 (return).
        // RPO is [b0, b1, b3, b2], so b2 comes after b3 -> term(b2) > term(b3).
        assert_eq!(iv1.last_use, b2_term, "v1 last_use");

        assert_eq!(iv2.def, b1_inst_pt, "v2 def");
        assert_eq!(iv2.last_use, b1_term, "v2 last_use");

        assert_eq!(iv3.def, b2_inst_pt, "v3 def");
        assert_eq!(iv3.last_use, b2_term, "v3 last_use");

        // Interference checks.
        assert!(intervals.interferes(v0, v1), "v0 and v1 both span from entry(b0)");
        assert!(intervals.interferes(v0, v2), "v0 spans loop, v2 within it");
        assert!(!intervals.interferes(v2, v3), "v2 and v3 in different blocks");
    }

    #[test]
    fn nested_loops() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 0)
          b1(v2: u32):
            v3 = lt v2, v0
            jmpif v3 then: b2(), else: b5()
          b2():
            jmp b3(u32 0)
          b3(v4: u32):
            v5 = lt v4, v1
            jmpif v5 then: b4(), else: b6()
          b4():
            v6 = add v4, u32 1
            jmp b3(v6)
          b5():
            return v2
          b6():
            v7 = add v2, u32 1
            jmp b1(v7)
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3, b4, b5, b6] = block_ids();
        assert_eq!(rpo, vec![b0, b1, b5, b2, b3, b6, b4]);

        let v0 = func.dfg[b0].parameters()[0]; // outer bound
        let v1 = func.dfg[b0].parameters()[1]; // inner bound
        let v2 = func.dfg[b1].parameters()[0]; // outer loop var
        let v4 = func.dfg[b3].parameters()[0]; // inner loop var
        let v3 = inst_result(func, b1, 0); // lt result in b1
        let v5 = inst_result(func, b3, 0); // lt result in b3
        let v6 = inst_result(func, b4, 0); // add result in b4
        let v7 = inst_result(func, b6, 0); // add result in b6

        let b1_inst = inst_id(func, b1, 0);
        let b3_inst = inst_id(func, b3, 0);
        let b4_inst = inst_id(func, b4, 0);
        let b6_inst = inst_id(func, b6, 0);

        let b0_entry = intervals.block_entry_point(b0).unwrap();
        let b2_entry = intervals.block_entry_point(b2).unwrap();
        let b1_inst_pt = intervals.instruction_point(b1_inst).unwrap();
        let b1_term = intervals.terminator_point(b1).unwrap();
        let b3_inst_pt = intervals.instruction_point(b3_inst).unwrap();
        let b3_term = intervals.terminator_point(b3).unwrap();
        let b4_inst_pt = intervals.instruction_point(b4_inst).unwrap();
        let b4_term = intervals.terminator_point(b4).unwrap();
        let b6_inst_pt = intervals.instruction_point(b6_inst).unwrap();
        let b6_term = intervals.terminator_point(b6).unwrap();

        let iv0 = intervals.get(v0).expect("v0 should have an interval");
        let iv1 = intervals.get(v1).expect("v1 should have an interval");
        let iv2 = intervals.get(v2).expect("v2 should have an interval");
        let iv3 = intervals.get(v3).expect("v3 should have an interval");
        let iv4 = intervals.get(v4).expect("v4 should have an interval");
        let iv5 = intervals.get(v5).expect("v5 should have an interval");
        let iv6 = intervals.get(v6).expect("v6 should have an interval");
        let iv7 = intervals.get(v7).expect("v7 should have an interval");

        // Exact interval assertions.
        assert_eq!(iv0.def, b0_entry, "v0 def");
        // v0 is live throughout both loops.
        // RPO is [b0, b1, b5, b2, b3, b6, b4], so b4 comes after b6 -> term(b4) > term(b6).
        assert_eq!(iv0.last_use, b4_term, "v0 last_use");

        assert_eq!(iv1.def, b0_entry, "v1 def");
        assert_eq!(iv1.last_use, b4_term, "v1 last_use");

        assert_eq!(iv2.def, b0_entry, "v2 def (idom adjusted to b0)");
        // v2 (outer loop header param) is live through b4 (inner loop) and
        // b6 (outer back-edge).
        assert_eq!(iv2.last_use, b4_term, "v2 last_use");

        assert_eq!(iv3.def, b1_inst_pt, "v3 def");
        assert_eq!(iv3.last_use, b1_term, "v3 last_use");

        assert_eq!(iv4.def, b2_entry, "v4 def (idom adjusted to b2)");
        // v4 (inner loop header param) is live through b4 (inner back-edge).
        assert_eq!(iv4.last_use, b4_term, "v4 last_use");

        assert_eq!(iv5.def, b3_inst_pt, "v5 def");
        assert_eq!(iv5.last_use, b3_term, "v5 last_use");

        assert_eq!(iv6.def, b4_inst_pt, "v6 def");
        assert_eq!(iv6.last_use, b4_term, "v6 last_use");

        assert_eq!(iv7.def, b6_inst_pt, "v7 def");
        assert_eq!(iv7.last_use, b6_term, "v7 last_use");

        // Interference checks.
        assert!(intervals.interferes(v0, v1), "v0 and v1 have identical ranges");
        assert!(intervals.interferes(v0, v4), "v0 spans entire function, v4 within inner loop");
        assert!(
            !intervals.interferes(v6, v7),
            "v6 in b4 and v7 in b6 — different blocks, no overlap"
        );
        assert!(
            !intervals.interferes(v3, v5),
            "v3 in b1 and v5 in b3 — different blocks, no overlap"
        );
        // v0 overlaps with every other value
        assert!(
            intervals.interferes(v0, v3)
                && intervals.interferes(v0, v5)
                && intervals.interferes(v0, v6)
                && intervals.interferes(v0, v7)
        );
    }

    #[test]
    fn block_param_idom_allocation() {
        // b3's parameters are allocated at their idom (b0).
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 27, Field 29)
          b2():
            jmp b3(Field 28, Field 40)
          b3(v1: Field, v2: Field):
            return v1
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3] = block_ids();
        assert_eq!(rpo, vec![b0, b2, b1, b3]);

        let v1 = func.dfg[b3].parameters()[0];
        let v2 = func.dfg[b3].parameters()[1];

        // v1 and v2 are params of b3, but their defs should be at b0's entry
        // (the idom of b3).
        let b0_entry = intervals.block_entry_point(b0).unwrap();

        let iv1 = intervals.get(v1).expect("v1 should have an interval");
        let iv2 = intervals.get(v2).expect("v2 should have an interval");

        assert_eq!(iv1.def, b0_entry, "v1 should be defined at idom (b0) entry");
        assert_eq!(iv2.def, b0_entry, "v2 should be defined at idom (b0) entry");
    }

    #[test]
    fn constants_in_diamond() {
        // A constant used in both branches of a diamond should be allocated
        // at the common dominator (b0), so its live interval def is extended
        // to b0's entry.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            jmpif v0 then: b1(), else: b2()
          b1():
            v2 = add v1, Field 42
            jmp b3(v2)
          b2():
            v3 = mul v1, Field 42
            jmp b3(v3)
          b3(v4: Field):
            return v4
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3] = block_ids();
        assert_eq!(rpo, vec![b0, b2, b1, b3]);

        let b0_entry = intervals.block_entry_point(b0).unwrap();

        let constants = ConstantAllocation::from_function(func);

        // Field 42 is used in both b1 and b2, so it should be allocated at b0
        // (the common dominator). It's the only constant in this function.
        let b0_constants = constants.allocated_in_block(b0);
        assert_eq!(
            b0_constants.len(),
            1,
            "exactly one constant (Field 42) should be allocated in b0"
        );
        let constant_id = b0_constants[0];

        // The constant's interval def should be at b0's entry.
        let iv = intervals.get(constant_id).expect("constant should have an interval");
        assert_eq!(iv.def, b0_entry, "Field 42 def should be at b0 entry (common dominator)");

        // RPO is [b0, b2, b1, b3], so b1 comes after b2.
        // Field 42's last_use should be at b1's instruction (the later use in RPO).
        let b1_inst_pt = intervals.instruction_point(inst_id(func, b1, 0)).unwrap();
        assert_eq!(
            iv.last_use, b1_inst_pt,
            "Field 42 last_use should be at b1's instruction (later in RPO)"
        );
    }

    #[test]
    fn constants_in_loop_hoisted() {
        // A constant used inside a loop body should be hoisted outside the loop.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(v1)
          b1(v2: u32):
            v3 = lt v2, v0
            jmpif v3 then: b2(), else: b3()
          b2():
            v4 = add v2, u32 10
            jmp b1(v4)
          b3():
            return v2
        }
        ";
        let (intervals, ssa) = build_intervals(src);
        let func = ssa.main();
        let rpo = PostOrder::with_function(func).into_vec_reverse();

        let [b0, b1, b2, b3] = block_ids();
        assert_eq!(rpo, vec![b0, b1, b3, b2]);

        let b0_entry = intervals.block_entry_point(b0).unwrap();

        let constants = ConstantAllocation::from_function(func);

        // u32 10 is used only inside the loop body (b2), but should be
        // hoisted to b0 (outside the loop). It's the only constant.
        let b0_constants = constants.allocated_in_block(b0);
        assert_eq!(b0_constants.len(), 1, "exactly one constant should be allocated in b0");
        let constant_id = b0_constants[0];

        // The constant's def should be at b0's entry (hoisted).
        let iv = intervals.get(constant_id).expect("u32 10 should have an interval");
        assert_eq!(iv.def, b0_entry, "u32 10 def should be at b0 entry (hoisted outside loop)");

        // The constant's last_use should be at b2's terminator: loop liveness
        // propagation keeps it live through the back-edge (b2 -> b1).
        let b2_term = intervals.terminator_point(b2).unwrap();
        assert_eq!(
            iv.last_use, b2_term,
            "u32 10 last_use should be at b2's terminator (loop back-edge)"
        );
    }
}

#[cfg(test)]
mod register_pressure {
    use super::tests::{block_ids, build_intervals};

    #[test]
    fn register_pressure_linear() {
        // Linear chain: each value dies before the next is used.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            return v2
        }
        ";
        let (intervals, _ssa) = build_intervals(src);

        let pressure = intervals.max_register_pressure();
        dbg!(pressure);
        // We have a value and result for an addition and two constants declared at the beginning of the block.
        assert_eq!(
            pressure, 4,
            "need at least 2 values for the adds and 2 for the constants, got {pressure}"
        );
    }

    #[test]
    fn register_pressure_linear_no_constants() {
        // Many values defined early, all used at the end -> high pressure.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            v3 = mul v0, v1
            v4 = add v2, v3
            v5 = mul v2, v3
            v6 = add v4, v5
            return v6
        }
        ";
        let (intervals, _ssa) = build_intervals(src);

        let pressure = intervals.max_register_pressure();
        // We have more values but we only ever need 4 registers at most
        // Two operands, a result, and the next instruction's result
        assert_eq!(pressure, 4, "got {pressure}");
    }

    #[test]
    fn register_pressure_at_block() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            jmpif v0 then: b1(), else: b2()
          b1():
            v2 = add v1, Field 1
            jmp b3(v2)
          b2():
            jmp b3(Field 42)
          b3(v3: Field):
            return v3
        }
        ";
        let (intervals, _ssa) = build_intervals(src);

        let [b0, _b1, _b2, b3] = block_ids();

        let p0 = intervals.register_pressure_at_block(b0);
        let p3 = intervals.register_pressure_at_block(b3);

        // b0 has more live values (v0, v1, plus b3's params allocated at idom).
        // b3 has fewer (just v3).
        assert_eq!(p0, 3, "got {p0}");
        assert_eq!(p3, 1, "got {p3}");
    }
}
