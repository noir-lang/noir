//! Detects consecutive `ArrayGet` → `MakeArray` patterns that can be replaced with
//! a single `mem_copy` during Brillig codegen.
//!
//! This is a read-only analysis computed once per function (in [`FunctionContext::new`])
//! and consumed during block codegen. It follows the same pattern as
//! [`ConstantAllocation`] and [`VariableLiveness`] — an analysis struct stored in
//! [`FunctionContext`], not a modification to the SSA IR.
//!
//! ## Pattern matched
//!
//! ```text
//! v_base = <dynamic index>
//! v0 = array_get(src, v_base)
//! v_idx1 = add(v_base, 1)
//! v1 = array_get(src, v_idx1)
//! ...
//! result = make_array([v0, v1, ...])
//! ```
//!
//! When matched, the `MakeArray` is replaced with a `mem_copy` from the source array,
//! and the individual `ArrayGet` instructions (plus their single-use index computations)
//! are skipped entirely during codegen.

use acvm::AcirField;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId},
    value::{Value, ValueId},
};

/// Minimum number of elements in a `MakeArray` to consider for memcpy optimization.
/// Small arrays don't benefit enough from the memcpy loop overhead.
const MIN_MEMCPY_ELEMENTS: usize = 8;

/// Per-function analysis identifying `MakeArray` instructions that can use `mem_copy`
/// and instructions whose codegen should be skipped.
#[derive(Default)]
pub(crate) struct MemcpyOptimizations {
    /// `MakeArray` instructions that should emit `mem_copy` instead of per-element stores.
    pub(crate) memcpy_groups: HashMap<InstructionId, MemcpyInfo>,
    /// Instructions whose Brillig codegen should be skipped entirely.
    /// Contains dead `ArrayGet`s and their single-use `Binary::Add` index computations.
    pub(crate) skip_instructions: HashSet<InstructionId>,
}

/// Information needed to emit a `mem_copy` for a `MakeArray` instruction.
pub(crate) struct MemcpyInfo {
    /// The source array to copy from.
    pub(crate) source_array: ValueId,
    /// The base index into the source array (dynamic, not a constant).
    pub(crate) base_index: ValueId,
    /// Number of elements to copy.
    pub(crate) length: usize,
}

impl MemcpyOptimizations {
    /// Analyze a function for memcpy optimization opportunities.
    pub(crate) fn from_function(func: &Function) -> Self {
        let dfg = &func.dfg;

        // Step 1: Build use counts for all values.
        let use_counts = build_use_counts(func);

        // Step 2: Scan for MakeArray instructions with the consecutive-get pattern.
        let mut result = Self::default();

        for block_id in func.reachable_blocks() {
            let block = &dfg[block_id];
            for &instruction_id in block.instructions() {
                if let Instruction::MakeArray { elements, .. } = &dfg[instruction_id] {
                    if elements.len() < MIN_MEMCPY_ELEMENTS {
                        continue;
                    }
                    let Some((source_array, base_index)) =
                        detect_consecutive_array_gets(elements, dfg)
                    else {
                        continue;
                    };

                    let length = elements.len();
                    result
                        .memcpy_groups
                        .insert(instruction_id, MemcpyInfo { source_array, base_index, length });

                    // Mark single-use array_gets and index computations for skipping.
                    // Element 0 is NOT skipped: its array_get naturally uses source_array
                    // and base_index in the SSA, which keeps them alive via liveness
                    // (plus the synthetic uses injected in VariableLiveness).
                    for (_i, element) in elements.iter().enumerate().skip(1) {
                        if use_counts.get(element).copied().unwrap_or(0) != 1 {
                            continue;
                        }
                        let Some(array_get_id) = defining_instruction(dfg, *element) else {
                            continue;
                        };
                        result.skip_instructions.insert(array_get_id);

                        // The index is produced by a Binary::Add. Skip if single-use.
                        if let Instruction::ArrayGet { index, .. } = &dfg[array_get_id]
                            && use_counts.get(index).copied().unwrap_or(0) == 1
                            && let Some(add_id) = defining_instruction(dfg, *index)
                        {
                            result.skip_instructions.insert(add_id);
                        }
                    }
                }
            }
        }
        result
    }
}

/// Build a map from ValueId to the number of times it appears as an operand
/// in instructions and terminators across all reachable blocks.
fn build_use_counts(func: &Function) -> HashMap<ValueId, u32> {
    let dfg = &func.dfg;
    let mut counts: HashMap<ValueId, u32> = HashMap::default();

    for block_id in func.reachable_blocks() {
        let block = &dfg[block_id];
        for &instruction_id in block.instructions() {
            dfg[instruction_id].for_each_value(|v| {
                *counts.entry(v).or_default() += 1;
            });
        }
        if let Some(terminator) = block.terminator() {
            terminator.for_each_value(|v| {
                *counts.entry(v).or_default() += 1;
            });
        }
    }
    counts
}

/// Check whether all elements of a `MakeArray` are `ArrayGet` instructions
/// from the same source array with consecutive dynamic indices.
///
/// Returns `Some((source_array, base_index))` on success.
fn detect_consecutive_array_gets(
    elements: &im::Vector<ValueId>,
    dfg: &DataFlowGraph,
) -> Option<(ValueId, ValueId)> {
    // Element 0 must be an ArrayGet with a non-constant (dynamic) index.
    let first = elements.front()?;
    let first_instr_id = defining_instruction(dfg, *first)?;
    let Instruction::ArrayGet { array: source, index: base_index } = &dfg[first_instr_id] else {
        return None;
    };

    // Only match dynamic base indices. Constant indices are already handled
    // efficiently by brillig_array_get_and_set (offset shifting).
    if dfg.get_numeric_constant(*base_index).is_some() {
        return None;
    }

    let source = *source;
    let base_index = *base_index;

    // Elements 1..N must be ArrayGet from the same source with index = base + constant(i).
    for (i, element) in elements.iter().enumerate().skip(1) {
        let instr_id = defining_instruction(dfg, *element)?;
        let Instruction::ArrayGet { array, index } = &dfg[instr_id] else {
            return None;
        };
        if *array != source {
            return None;
        }
        // The index must be `base_index + constant(i)`, produced by an unchecked add.
        if !is_base_plus_constant(dfg, *index, base_index, i as u128) {
            return None;
        }
    }

    Some((source, base_index))
}

/// Check if `index` is `base + constant(expected_offset)` via an unchecked Binary::Add.
fn is_base_plus_constant(
    dfg: &DataFlowGraph,
    index: ValueId,
    base: ValueId,
    expected_offset: u128,
) -> bool {
    let Some(instr_id) = defining_instruction(dfg, index) else {
        return false;
    };
    let Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Add { unchecked: true } }) =
        &dfg[instr_id]
    else {
        return false;
    };
    if *lhs != base {
        return false;
    }
    let Some(constant) = dfg.get_numeric_constant(*rhs) else {
        return false;
    };
    constant.to_u128() == expected_offset
}

/// Get the InstructionId that defines a given value, if it was produced by an instruction.
fn defining_instruction(dfg: &DataFlowGraph, value: ValueId) -> Option<InstructionId> {
    match &dfg[value] {
        Value::Instruction { instruction, .. } => Some(*instruction),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::ssa_gen::Ssa;

    use super::MemcpyOptimizations;

    fn analyze(src: &str) -> MemcpyOptimizations {
        let ssa = Ssa::from_str(src).unwrap();
        MemcpyOptimizations::from_function(ssa.main())
    }

    #[test]
    fn basic_consecutive_gets_detected() {
        // 8 consecutive array_gets from the same source with dynamic base + constant offsets.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 80], v1: u32):
            v2 = mul v1, u32 10
            v3 = array_get v0, index v2 -> Field
            v4 = unchecked_add v2, u32 1
            v5 = array_get v0, index v4 -> Field
            v6 = unchecked_add v2, u32 2
            v7 = array_get v0, index v6 -> Field
            v8 = unchecked_add v2, u32 3
            v9 = array_get v0, index v8 -> Field
            v10 = unchecked_add v2, u32 4
            v11 = array_get v0, index v10 -> Field
            v12 = unchecked_add v2, u32 5
            v13 = array_get v0, index v12 -> Field
            v14 = unchecked_add v2, u32 6
            v15 = array_get v0, index v14 -> Field
            v16 = unchecked_add v2, u32 7
            v17 = array_get v0, index v16 -> Field
            v18 = make_array [v3, v5, v7, v9, v11, v13, v15, v17] : [Field; 8]
            return v18
        }
        ";
        let opts = analyze(src);
        assert_eq!(opts.memcpy_groups.len(), 1, "should detect one memcpy group");
        // Elements 1..7: 7 array_gets + 7 Binary::Adds = 14 skipped.
        // Element 0 is NOT skipped.
        assert_eq!(opts.skip_instructions.len(), 14);
    }

    #[test]
    fn too_few_elements_not_detected() {
        // Only 4 elements — below MIN_MEMCPY_ELEMENTS threshold.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 40], v1: u32):
            v2 = mul v1, u32 4
            v3 = array_get v0, index v2 -> Field
            v4 = unchecked_add v2, u32 1
            v5 = array_get v0, index v4 -> Field
            v6 = unchecked_add v2, u32 2
            v7 = array_get v0, index v6 -> Field
            v8 = unchecked_add v2, u32 3
            v9 = array_get v0, index v8 -> Field
            v10 = make_array [v3, v5, v7, v9] : [Field; 4]
            return v10
        }
        ";
        let opts = analyze(src);
        assert!(opts.memcpy_groups.is_empty(), "should not detect memcpy for small arrays");
    }

    #[test]
    fn constant_index_not_detected() {
        // Base index is a constant — brillig_array_get_and_set handles these.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 80]):
            v1 = array_get v0, index u32 0 -> Field
            v2 = array_get v0, index u32 1 -> Field
            v3 = array_get v0, index u32 2 -> Field
            v4 = array_get v0, index u32 3 -> Field
            v5 = array_get v0, index u32 4 -> Field
            v6 = array_get v0, index u32 5 -> Field
            v7 = array_get v0, index u32 6 -> Field
            v8 = array_get v0, index u32 7 -> Field
            v9 = make_array [v1, v2, v3, v4, v5, v6, v7, v8] : [Field; 8]
            return v9
        }
        ";
        let opts = analyze(src);
        assert!(opts.memcpy_groups.is_empty(), "constant base index should not match");
    }

    #[test]
    fn multi_use_element_not_skipped() {
        // v5 (element 1) is used by both the make_array AND an add — should not be skipped.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 80], v1: u32):
            v2 = mul v1, u32 10
            v3 = array_get v0, index v2 -> Field
            v4 = unchecked_add v2, u32 1
            v5 = array_get v0, index v4 -> Field
            v6 = unchecked_add v2, u32 2
            v7 = array_get v0, index v6 -> Field
            v8 = unchecked_add v2, u32 3
            v9 = array_get v0, index v8 -> Field
            v10 = unchecked_add v2, u32 4
            v11 = array_get v0, index v10 -> Field
            v12 = unchecked_add v2, u32 5
            v13 = array_get v0, index v12 -> Field
            v14 = unchecked_add v2, u32 6
            v15 = array_get v0, index v14 -> Field
            v16 = unchecked_add v2, u32 7
            v17 = array_get v0, index v16 -> Field
            v18 = make_array [v3, v5, v7, v9, v11, v13, v15, v17] : [Field; 8]
            v19 = add v5, Field 1
            return v18
        }
        ";
        let opts = analyze(src);
        assert_eq!(opts.memcpy_groups.len(), 1, "memcpy group still detected");
        // Element 1 (v5) has 2 uses, so its array_get + Binary::Add are NOT skipped.
        // Element 0 is never skipped. Elements 2..7: 6 array_gets + 6 adds = 12.
        assert_eq!(opts.skip_instructions.len(), 12);
    }

    #[test]
    fn different_source_not_detected() {
        // Array_gets from two different source arrays — should not match.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 80], v1: [Field; 80], v2: u32):
            v3 = mul v2, u32 10
            v4 = array_get v0, index v3 -> Field
            v5 = unchecked_add v3, u32 1
            v6 = array_get v1, index v5 -> Field
            v7 = unchecked_add v3, u32 2
            v8 = array_get v0, index v7 -> Field
            v9 = unchecked_add v3, u32 3
            v10 = array_get v0, index v9 -> Field
            v11 = unchecked_add v3, u32 4
            v12 = array_get v0, index v11 -> Field
            v13 = unchecked_add v3, u32 5
            v14 = array_get v0, index v13 -> Field
            v15 = unchecked_add v3, u32 6
            v16 = array_get v0, index v15 -> Field
            v17 = unchecked_add v3, u32 7
            v18 = array_get v0, index v17 -> Field
            v19 = make_array [v4, v6, v8, v10, v12, v14, v16, v18] : [Field; 8]
            return v19
        }
        ";
        let opts = analyze(src);
        assert!(opts.memcpy_groups.is_empty(), "different sources should not match");
    }
}
