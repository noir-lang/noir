//! Detects consecutive `ArrayGet` → `MakeArray` patterns that can be replaced with
//! a single `mem_copy` during Brillig codegen.
//! It also detect consecutive `ArrayGet` written into consecutive registers,
//! and consecutive `ArraySet` of consecutive registers, which both can be optimized with `mem_copy`.
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

use acvm::{AcirField, FieldElement, acir::brillig::MemoryAddress};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    brillig::{
        brillig_gen::{brillig_block::BrilligBlock, brillig_block_variables::compute_array_length},
        brillig_ir::{
            BrilligBinaryOp, ReservedRegisters,
            brillig_variable::{
                BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
                get_bit_size_from_ssa_type,
            },
            registers::RegisterAllocator,
        },
    },
    ssa::ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
};

/// Minimum number of elements in a `MakeArray` to consider for memcpy optimization.
/// Small arrays don't benefit enough from the memcpy loop overhead.
const MIN_MEMCPY_ELEMENTS: usize = 4;

/// Per-function analysis identifying instructions that can use `mem_copy`
/// and instructions whose codegen should be skipped.
#[derive(Default)]
pub(crate) struct MemcpyOptimizations {
    /// `MakeArray` instructions that should emit `mem_copy` instead of per-element stores.
    pub(crate) memcpy_groups: HashMap<InstructionId, MemcpyInfo>,
    /// Consecutive `ArrayGet` runs that should emit a single `mem_copy` into
    /// consecutive stack registers. Keyed by the FIRST `ArrayGet`'s instruction ID.
    pub(crate) load_groups: HashMap<InstructionId, LoadGroupInfo>,
    /// Instructions whose Brillig codegen should be skipped entirely.
    /// Contains dead `ArrayGet`s and their single-use `Binary::Add` index computations.
    pub(crate) skip_instructions: HashSet<InstructionId>,
}

/// Information needed to emit a `mem_copy` for a `MakeArray` instruction.
pub(crate) struct MemcpyInfo {
    /// The source array to copy from.
    pub(crate) source_array: ValueId,
    /// The base index into the source array.
    pub(crate) base_index: ValueId,
    /// Number of elements to copy.
    pub(crate) length: usize,
}

/// Information needed to emit a `mem_copy` for a group of consecutive `ArrayGet` instructions.
#[derive(Clone)]
pub(crate) struct LoadGroupInfo {
    /// The source array to copy from.
    pub(crate) source_array: ValueId,
    /// The base index into the source array.
    pub(crate) base_index: ValueId,
    /// The instruction IDs of all consecutive array_gets in the group.
    pub(crate) array_get_ids: Vec<InstructionId>,
}

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// If the given registers are consecutive stack-relative addresses, try to emit a memcpy
    /// to copy them into `dest_pointer` (a heap array items pointer).
    ///
    /// Returns `true` if a memcpy was emitted (or patched), `false` if the registers
    /// aren't consecutive and the caller should fall back to individual stores.
    pub(super) fn try_memcpy_to_dest_from_consecutive_registers(
        &mut self,
        registers: &[MemoryAddress],
        dest_pointer: MemoryAddress,
    ) -> bool {
        let Some(first_offset) = try_get_consecutive_relative(registers) else {
            return false;
        };

        // Emit a memcpy from the consecutive stack registers to the heap destination.
        let source_ptr = self.brillig_context.allocate_register();
        let offset_const = self
            .brillig_context
            .make_usize_constant_instruction(FieldElement::from(u128::from(first_offset)));
        self.brillig_context.memory_op_instruction(
            ReservedRegisters::stack_pointer(),
            offset_const.address,
            *source_ptr,
            BrilligBinaryOp::Add,
        );
        let count = self
            .brillig_context
            .make_usize_constant_instruction(FieldElement::from(registers.len() as u128));
        self.brillig_context.call_mem_copy_procedure(*source_ptr, dest_pointer, count.address);
        true
    }

    /// Emits a single `MemCopy` call for a pre-detected load group, placing the
    /// results into consecutive stack registers and mapping each SSA result value.
    ///
    /// Called from `convert_ssa_instruction` when the current instruction is the
    /// first `ArrayGet` of a load group (detected by `from_function()`).
    pub(super) fn codegen_load_group(&mut self, info: &LoadGroupInfo, dfg: &DataFlowGraph) {
        let n = info.array_get_ids.len();

        // Allocate N consecutive destination registers.
        // ensure_register_capacity spills if needed to make room.
        self.ensure_register_capacity(n + 4);
        let dest_registers = self
            .brillig_context
            .allocate_consecutive_registers(n)
            .expect("ICE: not enough register space for load group after spilling");

        // Compute source pointer: array_base + base_index.
        let has_offset = dfg.get_numeric_constant(info.base_index).is_some();
        let array_variable = self.convert_ssa_value(info.source_array, dfg);
        let base_ptr = if has_offset {
            array_variable.extract_register()
        } else {
            *self.brillig_context.codegen_make_array_or_vector_items_pointer(array_variable)
        };

        let base_index_variable = self.convert_ssa_single_addr_value(info.base_index, dfg);
        let source_ptr = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            base_ptr,
            base_index_variable.address,
            *source_ptr,
            BrilligBinaryOp::Add,
        );

        // Compute destination pointer as absolute address: SP + dest_registers[0].offset
        let dest_ptr = self.brillig_context.allocate_register();
        let dest_relative_offset = dest_registers[0].unwrap_relative();
        let dest_offset_const = self
            .brillig_context
            .make_usize_constant_instruction(FieldElement::from(u128::from(dest_relative_offset)));
        self.brillig_context.memory_op_instruction(
            ReservedRegisters::stack_pointer(),
            dest_offset_const.address,
            *dest_ptr,
            BrilligBinaryOp::Add,
        );

        // Emit the memcpy call.
        let count =
            self.brillig_context.make_usize_constant_instruction(FieldElement::from(n as u128));
        self.brillig_context.call_mem_copy_procedure(*source_ptr, *dest_ptr, count.address);

        // Map each SSA result value to its pre-allocated consecutive register.
        for (i, &instruction_id) in info.array_get_ids.iter().enumerate() {
            let [result_id] = dfg.instruction_result(instruction_id);
            let typ = dfg.type_of_value(result_id);
            let register = dest_registers[i];
            let variable = make_variable_for_type(&typ, register);

            self.function_context.ssa_value_allocations.insert(result_id, variable);
            self.variables.add_available(result_id);
            if let Some(sm) = self.function_context.spill_manager.as_mut() {
                sm.touch(result_id);
            }
        }
    }
}

/// Constructs the appropriate `BrilligVariable` for a given SSA type using a pre-allocated register.
fn make_variable_for_type(typ: &Type, register: MemoryAddress) -> BrilligVariable {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => {
            let bit_size = get_bit_size_from_ssa_type(typ);
            BrilligVariable::SingleAddr(SingleAddrVariable::new(register, bit_size))
        }
        Type::Array(item_typ, elem_count) => {
            let size = compute_array_length(item_typ, *elem_count);
            BrilligVariable::BrilligArray(BrilligArray { pointer: register, size })
        }
        Type::Vector(_) => BrilligVariable::BrilligVector(BrilligVector { pointer: register }),
    }
}

/// Returns the relative offset of the first register if all registers are
/// consecutive `Relative` addresses (sp[K], sp[K+1], ...), or `None` otherwise.
fn try_get_consecutive_relative(registers: &[MemoryAddress]) -> Option<u32> {
    if registers.is_empty() || !registers[0].is_relative() {
        return None;
    }
    let first = registers[0].unwrap_relative();
    for (i, reg) in registers.iter().enumerate() {
        if !reg.is_relative() || reg.unwrap_relative() != first + i as u32 {
            return None;
        }
    }
    Some(first)
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

        // Step 3: Scan for standalone consecutive ArrayGet runs (not feeding into MakeArray).
        for block_id in func.reachable_blocks() {
            let block = &dfg[block_id];
            let instructions = block.instructions();
            let mut i = 0;
            while i < instructions.len() {
                // Skip instructions already handled by the MakeArray path.
                if result.skip_instructions.contains(&instructions[i]) {
                    i += 1;
                    continue;
                }

                let first_id = instructions[i];
                let (array, base_index) = match &dfg[first_id] {
                    Instruction::ArrayGet { array, index } => (*array, *index),
                    _ => {
                        i += 1;
                        continue;
                    }
                };

                // Scan forward for consecutive array_gets, skipping interleaved Binary::Add.
                let mut array_get_ids = vec![first_id];
                let mut skipped_ids = Vec::new();
                for &inst_id in &instructions[(i + 1)..] {
                    if result.skip_instructions.contains(&inst_id) {
                        break;
                    }
                    match &dfg[inst_id] {
                        Instruction::ArrayGet { array: a, index } if *a == array => {
                            let expected_offset = array_get_ids.len() as u128;
                            if is_index_base_plus_offset(dfg, base_index, *index, expected_offset) {
                                array_get_ids.push(inst_id);
                                continue;
                            }
                            break;
                        }
                        Instruction::ArrayGet { .. } => break,
                        Instruction::Binary(_) => {
                            let [result_id] = dfg.instruction_result(inst_id);
                            let expected_offset = array_get_ids.len() as u128;
                            if is_index_base_plus_offset(
                                dfg,
                                base_index,
                                result_id,
                                expected_offset,
                            ) {
                                skipped_ids.push(inst_id);
                                continue;
                            }
                            break;
                        }
                        _ => break,
                    }
                }

                if array_get_ids.len() < MIN_MEMCPY_ELEMENTS {
                    i += 1;
                    continue;
                }

                let total_consumed = array_get_ids.len() + skipped_ids.len();

                // Mark elements 1..N and their single-use index computations for skipping.
                for &get_id in &array_get_ids[1..] {
                    let [result_id] = dfg.instruction_result(get_id);
                    if use_counts.get(&result_id).copied().unwrap_or(0) <= 1 {
                        result.skip_instructions.insert(get_id);
                    }
                }
                for &add_id in &skipped_ids {
                    let [result_id] = dfg.instruction_result(add_id);
                    if use_counts.get(&result_id).copied().unwrap_or(0) <= 1 {
                        result.skip_instructions.insert(add_id);
                    }
                }
                result.load_groups.insert(
                    first_id,
                    LoadGroupInfo {
                        source_array: array,
                        base_index,
                        array_get_ids: array_get_ids.clone(),
                    },
                );

                i += total_consumed;
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
    // Element 0 must be an ArrayGet.
    let first = elements.front()?;
    let first_instr_id = defining_instruction(dfg, *first)?;
    let Instruction::ArrayGet { array: source, index: base_index } = &dfg[first_instr_id] else {
        return None;
    };

    let source = *source;
    let base_index = *base_index;

    // Elements 1..N must be ArrayGet from the same source with index = base + i.
    for (i, element) in elements.iter().enumerate().skip(1) {
        let instr_id = defining_instruction(dfg, *element)?;
        let Instruction::ArrayGet { array, index } = &dfg[instr_id] else {
            return None;
        };
        if *array != source {
            return None;
        }
        if !is_index_base_plus_offset(dfg, base_index, *index, i as u128) {
            return None;
        }
    }

    Some((source, base_index))
}

/// Check if `candidate` equals `base + expected_offset` in the SSA value graph.
///
/// Handles two cases:
/// 1. Both are numeric constants: `candidate_const - base_const == expected_offset`
/// 2. `candidate` is defined by `Binary::Add(base, constant(expected_offset))`
fn is_index_base_plus_offset(
    dfg: &DataFlowGraph,
    base: ValueId,
    candidate: ValueId,
    expected_offset: u128,
) -> bool {
    // Case 1: both are constants — compare their numeric values.
    if let (Some(base_const), Some(cand_const)) =
        (dfg.get_numeric_constant(base), dfg.get_numeric_constant(candidate))
    {
        return cand_const.to_u128() == base_const.to_u128() + expected_offset;
    }

    // Case 2: candidate = base + constant(expected_offset) via Binary::Add.
    let Some(instr_id) = defining_instruction(dfg, candidate) else {
        return false;
    };
    let Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Add { .. } }) = &dfg[instr_id]
    else {
        return false;
    };
    // Check lhs=base, rhs=constant(expected_offset)
    if *lhs == base
        && let Some(c) = dfg.get_numeric_constant(*rhs)
    {
        return c.to_u128() == expected_offset;
    }
    // Check rhs=base, lhs=constant(expected_offset)
    if *rhs == base
        && let Some(c) = dfg.get_numeric_constant(*lhs)
    {
        return c.to_u128() == expected_offset;
    }
    false
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
    use acvm::acir::brillig::Opcode as BrilligOpcode;

    use crate::{
        brillig::{
            BrilligOptions,
            brillig_gen::{brillig_fn::FunctionContext, gen_brillig_for},
            brillig_ir::artifact::GeneratedBrillig,
        },
        ssa::ssa_gen::Ssa,
    };

    use super::MemcpyOptimizations;

    fn analyze(src: &str) -> MemcpyOptimizations {
        let ssa = Ssa::from_str(src).unwrap();
        MemcpyOptimizations::from_function(ssa.main())
    }

    fn compile_ssa_to_brillig(src: &str) -> GeneratedBrillig<acvm::FieldElement> {
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(&BrilligOptions::default());
        let func = ssa.main();
        let arguments: Vec<_> = func
            .parameters()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                FunctionContext::ssa_type_to_parameter(&typ)
            })
            .collect();
        gen_brillig_for(func, arguments, &brillig, &BrilligOptions::default()).unwrap()
    }

    /// Count the number of `Call` instructions in the bytecode (memcpy procedure calls).
    fn count_calls(generated: &GeneratedBrillig<acvm::FieldElement>) -> usize {
        generated.byte_code.iter().filter(|op| matches!(op, BrilligOpcode::Call { .. })).count()
    }

    /// Count the number of `Store` instructions in the bytecode.
    fn count_stores(generated: &GeneratedBrillig<acvm::FieldElement>) -> usize {
        generated.byte_code.iter().filter(|op| matches!(op, BrilligOpcode::Store { .. })).count()
    }

    /// Count `Load` instructions (each individual array_get emits one).
    fn count_loads(generated: &GeneratedBrillig<acvm::FieldElement>) -> usize {
        generated.byte_code.iter().filter(|op| matches!(op, BrilligOpcode::Load { .. })).count()
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
        // Only 3 elements — below MIN_MEMCPY_ELEMENTS threshold.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 40], v1: u32):
            v2 = mul v1, u32 4
            v3 = array_get v0, index v2 -> Field
            v4 = unchecked_add v2, u32 1
            v5 = array_get v0, index v4 -> Field
            v6 = unchecked_add v2, u32 2
            v7 = array_get v0, index v6 -> Field
            v10 = make_array [v3, v5, v7] : [Field; 3]
            return v10
        }
        ";
        let opts = analyze(src);
        assert!(opts.memcpy_groups.is_empty(), "should not detect memcpy for small arrays");
    }

    #[test]
    fn constant_index_detected() {
        // Consecutive constant indices should also be detected.
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
        assert_eq!(opts.memcpy_groups.len(), 1, "should detect constant index pattern");
        // All array_gets except element 0 should be skipped (7 array_gets, no index adds).
        assert_eq!(opts.skip_instructions.len(), 7);
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

        let generated = compile_ssa_to_brillig(src);
        let calls = count_calls(&generated);
        assert!(calls == 5, "Expected 5 Call instructions (including memcpy calls), got {calls}");

        // The make_array should NOT have individual stores (the 8 stores are replaced by memcpy).
        // The only stores remaining should be from the MemCopy procedure body itself.
        let stores = count_stores(&generated);
        assert!(stores == 1, "Expected 1 Store instruction (in MemCopy procedure), got {stores}");

        let loads = count_loads(&generated);
        assert!(
            loads == 3,
            "Expected 3 Load instructions (1 in MemCopy + 2 non-skipped array_gets), got {loads}"
        );
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

    #[test]
    fn load_optimization() {
        // All 8 array_get results are used twice: once by make_array, once by an add.
        // The MakeArray path skips none of them (all have use_count > 1).
        // The Load group analysis should detect all 8 consecutive array_gets and emit a memcpy
        // into consecutive registers.
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
            v19 = add v3, Field 1
            v20 = add v5, Field 2
            v21 = add v7, Field 3
            v22 = add v9, Field 4
            v23 = add v11, Field 5
            v24 = add v13, Field 6
            v25 = add v15, Field 7
            v26 = add v17, Field 8
            return v18
        }
        ";
        let opts = analyze(src);

        // MakeArray is still detected (all elements are consecutive array_gets).
        assert_eq!(opts.memcpy_groups.len(), 1, "MakeArray memcpy group detected");

        // No array_gets are skipped by the MakeArray path (all have use_count > 1).
        // But then Load group should detect 8 consecutive array_gets.
        assert_eq!(opts.load_groups.len(), 1, "Load group should be detected");
        let load_group = opts.load_groups.values().next().unwrap();
        assert_eq!(load_group.array_get_ids.len(), 8, "Should have 8 array_gets in the group");

        // The skipped instructions should include the 7 interleaved unchecked_adds
        // (index computations for elements 1-7, all single-use).
        assert_eq!(opts.skip_instructions.len(), 7, "7 index adds should be skipped");
    }

    #[test]
    fn store_optimization() {
        // 8 binary operations whose results are allocated consecutively by the register
        // allocator (no deallocations between them), then fed into a make_array.
        // MakeArray is doing consecutive array_sets which can be folded in a memcpy.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v0, Field 2
            v3 = add v0, Field 3
            v4 = add v0, Field 4
            v5 = add v0, Field 5
            v6 = add v0, Field 6
            v7 = add v0, Field 7
            v8 = add v0, Field 8
            v9 = make_array [v1, v2, v3, v4, v5, v6, v7, v8] : [Field; 8]
            return v9
        }
        ";
        let opts = analyze(src);

        // No memcpy_group (elements are not array_gets).
        assert!(opts.memcpy_groups.is_empty(), "Should not detect MakeArray memcpy group");
        // No load group (no array_gets at all).
        assert!(opts.load_groups.is_empty(), "Should not detect load group");

        let generated = compile_ssa_to_brillig(src);

        // The 8 individual stores are replaced by a memcpy call.
        // We'd see at most 1 Store (in the MemCopy procedure) instead of 8.
        let stores = count_stores(&generated);
        assert!(stores == 1, "Expected at most 1 Store (in MemCopy procedure), got {stores}");
    }

    #[test]
    fn store_optimization_with_constants() {
        // MakeArray is doing consecutive array_sets which can be folded in a memcpy, even with constant sources.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v9 = make_array [Field 1,Field 2,Field 3,Field 4,Field 5,Field 6,Field 7,Field 8] : [Field; 8]
            return v9
        }
        ";
        let opts = analyze(src);

        // No memcpy_group (elements are not array_gets).
        assert!(opts.memcpy_groups.is_empty(), "Should not detect MakeArray memcpy group");
        // No load group (no array_gets at all).
        assert!(opts.load_groups.is_empty(), "Should not detect load group");

        let generated = compile_ssa_to_brillig(src);

        // The 8 individual stores are replaced by a memcpy call.
        // We'd see at most 1 Store (in the MemCopy procedure) instead of 8.
        let stores = count_stores(&generated);
        assert!(stores == 1, "Expected at most 1 Store (in MemCopy procedure), got {stores}");
    }
}
