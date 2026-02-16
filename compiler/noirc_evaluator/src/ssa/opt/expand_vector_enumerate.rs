//! This module implements the expansion of `vector_enumerate` intrinsic calls.
//!
//! The `vector_enumerate` intrinsic takes a vector and a closure, calling the closure
//! for each index with (vec, iteration_index, user_index) as arguments. The closure returns a T value
//! which is written to the output vector at that index via ArraySet. The final modified vector is returned.
//!
//! This expansion only applies:
//! - In ACIR runtime (not Brillig)
//! - When the vector length is a compile-time constant
//!
//! This pass should run after:
//! - The main inlining pass (so closures are already available)
//! - The as_vector_optimization pass (so vector lengths are constants)

use acvm::{AcirField, FieldElement};

use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        types::{NumericType, Type},
        value::{Value, ValueId, ValueMapping},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Expand `vector_enumerate` calls in ACIR functions where the vector length is constant.
    /// Closure arguments remain as Value::Function (defunctionalization skips them),
    /// so we look up the post-flatten functions directly from self.functions.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn expand_vector_enumerate(mut self) -> Self {
        // Collect closure FunctionIds referenced by vector_enumerate calls across all ACIR functions,
        // then clone those (post-flatten, single-block) functions for use during inlining.
        let mut closure_funcs: std::collections::BTreeMap<FunctionId, Function> =
            std::collections::BTreeMap::new();
        for func in self.functions.values() {
            if !func.runtime().is_acir() {
                continue;
            }
            for block_id in func.reachable_blocks() {
                for &instr_id in func.dfg[block_id].instructions() {
                    if let Instruction::Call { func: call_func, arguments } = &func.dfg[instr_id] {
                        if matches!(
                            &func.dfg[*call_func],
                            Value::Intrinsic(Intrinsic::VectorEnumerate)
                        ) {
                            if arguments.len() == 6 {
                                if let Value::Function(id) = &func.dfg[arguments[2]] {
                                    if let Some(f) = self.functions.get(id) {
                                        closure_funcs.insert(*id, f.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for func in self.functions.values_mut() {
            if func.runtime().is_acir() {
                func.expand_vector_enumerate(&closure_funcs);
            }
        }

        // After expanding vector_enumerate calls, simplify resize_array calls.
        // resize_array is not supported by the ACIR backend, so we lower it
        // to make_array + array_gets when the capacity is known.
        // Run twice: the first pass may resolve capacities that enable
        // subsequent resize_array calls to be simplified.
        for _ in 0..2 {
            for func in self.functions.values_mut() {
                if func.runtime().is_acir() {
                    func.simplify_resize_array_calls();
                }
            }
        }

        self
    }
}

impl Function {
    /// Expand `vector_enumerate` calls where the vector length is a compile-time constant.
    pub(crate) fn expand_vector_enumerate(
        &mut self,
        closure_functions: &std::collections::BTreeMap<FunctionId, Function>,
    ) {
        let Some(vector_enumerate) = self.dfg.get_intrinsic(Intrinsic::VectorEnumerate).copied()
        else {
            return;
        };

        let blocks: Vec<BasicBlockId> = self.reachable_blocks().into_iter().collect();

        for block_id in blocks {
            let instructions: Vec<InstructionId> = self.dfg[block_id].instructions().to_vec();

            for instruction_id in instructions {
                let instruction = self.dfg[instruction_id].clone();

                let (func, arguments) = match &instruction {
                    Instruction::Call { func, arguments } => (*func, arguments.clone()),
                    _ => continue,
                };

                if func != vector_enumerate {
                    continue;
                }

                self.expand_enumerate_call(block_id, instruction_id, &arguments, closure_functions);
            }
        }
    }

    /// Expand a single vector_enumerate call into closure calls + ArraySet chain.
    /// Closure args are Value::Function (not defunctionalized), so we look up the
    /// post-flatten function directly.
    fn expand_enumerate_call(
        &mut self,
        block_id: BasicBlockId,
        instruction_id: InstructionId,
        arguments: &[ValueId],
        closure_functions: &std::collections::BTreeMap<FunctionId, Function>,
    ) {
        // vector_enumerate has 6 SSA arguments:
        // length, vec, acir_closure, brillig_closure, index, rev
        if arguments.len() != 6 {
            return;
        }

        let length_value = arguments[0];
        let vector_value = arguments[1];
        let closure_arg = arguments[2]; // ACIR closure
        let user_index = arguments[4];
        let rev_value = arguments[5];

        // Resolve the vector length as iteration count.
        // Track whether length came from a constant resolution or from data capacity,
        // because when lengths are runtime values (e.g. after if-else merge) we must NOT
        // replace them with compile-time constants in the result mapping.
        let (length, length_is_constant) = if let Some(l) = self.resolve_length(length_value) {
            (l, true)
        } else if let Some(cap) = self.resolve_data_capacity(vector_value) {
            (cap, false)
        } else if let Some(cap) = self.dfg.try_get_vector_capacity(vector_value) {
            (cap.0 as usize, false)
        } else {
            return;
        };

        // Get rev as compile-time constant
        let rev = match self.dfg.get_numeric_constant(rev_value) {
            Some(r) => !r.is_zero(),
            None => return,
        };

        // Look up the closure function for manual inlining.
        // Closure args remain as Value::Function (defunctionalization skips them).
        let stored_closure = if let Value::Function(func_id) = &self.dfg[closure_arg] {
            closure_functions.get(func_id)
        } else {
            None
        };

        // Get the element type from the vector's type
        let vec_type = self.dfg.type_of_value(vector_value);
        let element_type = match &vec_type {
            Type::Vector(element_types) => element_types[0].clone(),
            _ => return,
        };

        let call_stack = self.dfg.get_instruction_call_stack_id(instruction_id);

        // Get the original instruction's result IDs before removing it
        let original_results = self.dfg.instruction_results(instruction_id).to_vec();

        // Remove vector_enumerate and save all instructions that come AFTER it.
        // The expansion will be appended at the end of the block (at the position
        // where vector_enumerate was), then the saved instructions are re-added.
        // This ensures the expansion's results are defined before they're used.
        let after_instructions = {
            let instructions = self.dfg[block_id].instructions_mut();
            let pos = instructions.iter().position(|&id| id == instruction_id).unwrap();
            let after = instructions[pos + 1..].to_vec();
            instructions.truncate(pos);
            after
        };

        // Iteration order
        let indices: Vec<usize> =
            if rev { (0..length).rev().collect() } else { (0..length).collect() };

        // Create a zeroed starting array for the ArraySet chain.
        // All positions will be overwritten by the expansion, so initial values don't matter.
        let zero = match &element_type {
            Type::Numeric(nt) => self.dfg.make_constant(FieldElement::zero(), *nt),
            _ => {
                // For non-numeric element types, keep using the original vector
                vector_value
            }
        };
        let mut current_vec = if matches!(&element_type, Type::Numeric(_)) {
            let elements: im::Vector<ValueId> = (0..length).map(|_| zero).collect();
            let make_arr = Instruction::MakeArray { elements, typ: vec_type.clone() };
            self.dfg
                .insert_instruction_and_results_without_simplification(
                    make_arr, block_id, None, call_stack,
                )
                .first()
        } else {
            vector_value
        };

        for i in indices {
            let index =
                self.dfg.make_constant(FieldElement::from(i as u128), NumericType::unsigned(32));

            let call_args = vec![length_value, vector_value, index, user_index];

            let result_value = if let Some(closure_func) = stored_closure {
                // Manually inline the post-flatten closure (single-block after flatten_cfg)
                self.inline_closure_call(
                    block_id,
                    closure_func,
                    &call_args,
                    &element_type,
                    call_stack,
                )
            } else {
                // Direct function call for non-defunctionalized closures
                let call = Instruction::Call { func: closure_arg, arguments: call_args };
                self.dfg
                    .insert_instruction_and_results_without_simplification(
                        call,
                        block_id,
                        Some(vec![element_type.clone()]),
                        call_stack,
                    )
                    .first()
            };

            // ArraySet: write the closure result to current_vec at this index
            let array_set = Instruction::ArraySet {
                array: current_vec,
                index,
                value: result_value,
                mutable: false,
            };
            let set_result = self.dfg.insert_instruction_and_results_without_simplification(
                array_set, block_id, None, call_stack,
            );
            current_vec = set_result.first();
        }

        // Re-add the instructions that came after vector_enumerate
        self.dfg[block_id].instructions_mut().extend(after_instructions);

        // Map original results to expansion results using ValueMapping.
        // When lengths are compile-time constants, also replace resize_array length
        // results with constants so the resize_array becomes dead.
        // When lengths are runtime values (e.g. from if-else merge), we must NOT
        // replace them with constants — only the data result gets remapped.
        if original_results.len() >= 2 {
            let mut mapping = ValueMapping::default();
            // Replace vector_enumerate data result with the ArraySet chain
            mapping.insert(original_results[1], current_vec);

            if length_is_constant {
                let const_len = self
                    .dfg
                    .make_constant(FieldElement::from(length as u128), NumericType::unsigned(32));
                // Replace vector_enumerate length result with constant
                mapping.insert(original_results[0], const_len);
                // If length_value came from a resize_array (not already a constant),
                // also replace it with the constant so the resize_array becomes dead
                if length_value != const_len
                    && self.dfg.get_numeric_constant(length_value).is_none()
                {
                    mapping.insert(length_value, const_len);
                }
            } else {
                // Lengths are runtime — map vector_enumerate's length result
                // to the runtime length (which is the correct output length)
                mapping.insert(original_results[0], length_value);
            }
            self.dfg.replace_values_in_block(block_id, &mapping);
        }
    }

    /// Manually inline a stored closure function into the current block.
    /// The closure must be single-block (post-flatten_cfg).
    /// Maps function parameters to call_args, copies all instructions, and returns
    /// the value that the closure's return terminator produces.
    ///
    /// Handles `enable_side_effects` correctly: the closure's internal conditions
    /// are ANDed with the outer side-effects condition so that inlining into a
    /// conditional context doesn't break the outer condition.
    fn inline_closure_call(
        &mut self,
        block_id: BasicBlockId,
        closure_func: &Function,
        call_args: &[ValueId],
        _element_type: &Type,
        call_stack: CallStackId,
    ) -> ValueId {
        let entry_block = closure_func.entry_block();
        let closure_params = closure_func.dfg[entry_block].parameters().to_vec();
        let closure_instructions = closure_func.dfg[entry_block].instructions().to_vec();

        // Find the outer side-effects condition by scanning back through the block's
        // instructions for the most recent EnableSideEffectsIf.
        let outer_condition = {
            let mut cond = None;
            for &instr_id in self.dfg[block_id].instructions().iter().rev() {
                if let Instruction::EnableSideEffectsIf { condition } = &self.dfg[instr_id] {
                    cond = Some(*condition);
                    break;
                }
            }
            cond
        };

        // Map closure parameters -> call arguments.
        // Dead parameter pruning may have removed unused params, so we match by type
        // instead of simple positional zip.
        let mut value_map: rustc_hash::FxHashMap<ValueId, ValueId> =
            rustc_hash::FxHashMap::default();
        let mut arg_idx = 0;
        for param in closure_params.iter() {
            let param_type = closure_func.dfg.type_of_value(*param);
            // Advance through call_args until we find one matching this param's type
            while arg_idx < call_args.len() {
                let arg_type = self.dfg.type_of_value(call_args[arg_idx]);
                if arg_type == param_type {
                    value_map.insert(*param, call_args[arg_idx]);
                    arg_idx += 1;
                    break;
                }
                arg_idx += 1;
            }
        }

        // Pre-populate value_map with all constants used in the closure's instructions
        // and terminator. This avoids needing &mut self.dfg during map_values.
        for &instr_id in &closure_instructions {
            closure_func.dfg[instr_id].for_each_value(|v| {
                if !value_map.contains_key(&v) {
                    if let Value::NumericConstant { constant, typ } = &closure_func.dfg[v] {
                        let new_v = self.dfg.make_constant(*constant, *typ);
                        value_map.insert(v, new_v);
                    }
                }
            });
        }
        let terminator = closure_func.dfg[entry_block].unwrap_terminator();
        terminator.for_each_value(|v| {
            if !value_map.contains_key(&v) {
                if let Value::NumericConstant { constant, typ } = &closure_func.dfg[v] {
                    let new_v = self.dfg.make_constant(*constant, *typ);
                    value_map.insert(v, new_v);
                }
            }
        });

        // Copy each instruction from the closure into our block
        let one = self.dfg.make_constant(FieldElement::one(), NumericType::bool());
        for &instr_id in &closure_instructions {
            let instruction = closure_func.dfg[instr_id].clone();
            let instruction = instruction.map_values(|v| value_map[&v]);

            // Handle enable_side_effects: make conditions relative to outer context
            let instruction = if let Instruction::EnableSideEffectsIf { condition } = &instruction {
                if let Some(outer_cond) = outer_condition {
                    if *condition == one {
                        // `enable_side_effects u1 1` (restore) → restore to outer condition
                        Instruction::EnableSideEffectsIf { condition: outer_cond }
                    } else {
                        // `enable_side_effects X` → `enable_side_effects (X AND outer_cond)`
                        let and_result =
                            self.dfg.insert_instruction_and_results_without_simplification(
                                Instruction::binary(BinaryOp::And, *condition, outer_cond),
                                block_id,
                                None,
                                call_stack,
                            );
                        Instruction::EnableSideEffectsIf { condition: and_result.first() }
                    }
                } else {
                    instruction
                }
            } else {
                instruction
            };

            // Get result types from the closure's instruction results
            let result_types: Vec<Type> = closure_func
                .dfg
                .instruction_results(instr_id)
                .iter()
                .map(|r| closure_func.dfg.type_of_value(*r))
                .collect();

            let new_results = self.dfg.insert_instruction_and_results_without_simplification(
                instruction,
                block_id,
                Some(result_types),
                call_stack,
            );

            // Map old result IDs to new ones
            let old_results = closure_func.dfg.instruction_results(instr_id);
            for (old_r, new_r) in old_results.iter().zip(new_results.results().iter()) {
                value_map.insert(*old_r, *new_r);
            }
        }

        // After inlining, restore the outer side-effects condition
        if let Some(outer_cond) = outer_condition {
            self.dfg.insert_instruction_and_results_without_simplification(
                Instruction::EnableSideEffectsIf { condition: outer_cond },
                block_id,
                None,
                call_stack,
            );
        }

        // Get the return value from the closure's terminator
        match terminator {
            TerminatorInstruction::Return { return_values, .. } => {
                if let Some(&ret_val) = return_values.first() {
                    value_map[&ret_val]
                } else {
                    self.dfg.make_constant(FieldElement::zero(), NumericType::NativeField)
                }
            }
            _ => self.dfg.make_constant(FieldElement::zero(), NumericType::NativeField),
        }
    }
    /// Simplify resize_array calls into make_array + array_gets.
    /// resize_array is not supported by the ACIR backend, so we lower it
    /// to make_array + array_gets when the backing array capacity is known.
    ///
    /// The capacity is resolved from either:
    /// 1. The length value (when it's a compile-time constant), or
    /// 2. The data array itself (tracing through MakeArray/ArraySet chains)
    ///
    /// When the length is a runtime value (e.g. after an if-else merge),
    /// the output length is computed at runtime via Add/Sub instructions.
    fn simplify_resize_array_calls(&mut self) {
        let Some(resize_array) = self.dfg.get_intrinsic(Intrinsic::ResizeArray).copied() else {
            return;
        };

        let blocks: Vec<BasicBlockId> = self.reachable_blocks().into_iter().collect();

        for block_id in blocks {
            let instructions: Vec<InstructionId> = self.dfg[block_id].instructions().to_vec();

            for instruction_id in instructions {
                let instruction = self.dfg[instruction_id].clone();
                let (func, arguments) = match &instruction {
                    Instruction::Call { func, arguments } => (*func, arguments.clone()),
                    _ => continue,
                };

                if func != resize_array {
                    continue;
                }

                // resize_array(len, data, adjust) -> (u32, [T])
                if arguments.len() != 3 {
                    continue;
                }

                let input_len_value = arguments[0];
                let input_data = arguments[1];
                let adjust_value = arguments[2];

                // Resolve the backing array capacity: first from the length value
                // (when it's a constant), then by tracing the data array itself.
                let resolved_len = self.resolve_length(input_len_value);
                let input_capacity = if let Some(len) = resolved_len {
                    len
                } else if let Some(cap) = self.resolve_data_capacity(input_data) {
                    cap
                } else {
                    continue;
                };
                let Some(adjust_field) = self.dfg.get_numeric_constant(adjust_value) else {
                    continue;
                };
                let adjust = adjust_field.to_i128() as i32;
                let output_capacity = (input_capacity as i32 + adjust) as usize;

                // Get element type from the data vector
                let data_type = self.dfg.type_of_value(input_data);
                let element_type = match &data_type {
                    Type::Vector(element_types) => element_types[0].clone(),
                    _ => continue,
                };

                let call_stack = self.dfg.get_instruction_call_stack_id(instruction_id);
                let original_results = self.dfg.instruction_results(instruction_id).to_vec();

                // Remove resize_array and save instructions after it (same ordering fix
                // as vector_enumerate expansion — new instructions are appended to end of
                // block, so we need to put them at the original position).
                let after_instructions = {
                    let instructions = self.dfg[block_id].instructions_mut();
                    let pos = instructions.iter().position(|&id| id == instruction_id).unwrap();
                    let after = instructions[pos + 1..].to_vec();
                    instructions.truncate(pos);
                    after
                };

                // Build a make_array with array_gets from input + zeros for new positions
                let mut elements: im::Vector<ValueId> = im::Vector::new();
                // Copy elements from input (up to min(input_capacity, output_capacity))
                let copy_count = input_capacity.min(output_capacity);
                for i in 0..copy_count {
                    let idx = self
                        .dfg
                        .make_constant(FieldElement::from(i as u128), NumericType::unsigned(32));
                    let array_get = Instruction::ArrayGet { array: input_data, index: idx };
                    let elem = self.dfg.insert_instruction_and_results_without_simplification(
                        array_get,
                        block_id,
                        Some(vec![element_type.clone()]),
                        call_stack,
                    );
                    elements.push_back(elem.first());
                }
                // Add zeroed elements for growing
                if output_capacity > input_capacity {
                    let zero = match &element_type {
                        Type::Numeric(nt) => self.dfg.make_constant(FieldElement::zero(), *nt),
                        _ => continue,
                    };
                    for _ in input_capacity..output_capacity {
                        elements.push_back(zero);
                    }
                }

                let make_arr = Instruction::MakeArray { elements, typ: data_type.clone() };
                let new_data_value = self
                    .dfg
                    .insert_instruction_and_results_without_simplification(
                        make_arr, block_id, None, call_stack,
                    )
                    .first();

                // Compute the output length: constant if input length was resolved,
                // otherwise a runtime Add/Sub instruction.
                let new_len_value = if let Some(input_len) = resolved_len {
                    let output_len = (input_len as i32 + adjust) as usize;
                    self.dfg.make_constant(
                        FieldElement::from(output_len as u128),
                        NumericType::unsigned(32),
                    )
                } else if adjust >= 0 {
                    let adjust_const = self.dfg.make_constant(
                        FieldElement::from(adjust as u128),
                        NumericType::unsigned(32),
                    );
                    self.dfg
                        .insert_instruction_and_results_without_simplification(
                            Instruction::binary(
                                BinaryOp::Add { unchecked: false },
                                input_len_value,
                                adjust_const,
                            ),
                            block_id,
                            None,
                            call_stack,
                        )
                        .first()
                } else {
                    let adjust_const = self.dfg.make_constant(
                        FieldElement::from((-adjust) as u128),
                        NumericType::unsigned(32),
                    );
                    self.dfg
                        .insert_instruction_and_results_without_simplification(
                            Instruction::binary(
                                BinaryOp::Sub { unchecked: false },
                                input_len_value,
                                adjust_const,
                            ),
                            block_id,
                            None,
                            call_stack,
                        )
                        .first()
                };

                // Re-add saved instructions
                self.dfg[block_id].instructions_mut().extend(after_instructions);

                // Replace results: length → new length, data → make_array
                if original_results.len() >= 2 {
                    let mut mapping = ValueMapping::default();
                    mapping.insert(original_results[0], new_len_value);
                    mapping.insert(original_results[1], new_data_value);
                    self.dfg.replace_values_in_block(block_id, &mapping);
                }
            }
        }
    }

    /// Resolve a value to a constant length. Handles:
    /// - Direct numeric constants
    /// - Results of resize_array(len, data, adjust) → len + adjust (recursive)
    /// - Results of vector_enumerate → maps to the already-expanded length
    fn resolve_length(&self, value_id: ValueId) -> Option<usize> {
        // Direct constant
        if let Some(l) = self.dfg.get_numeric_constant(value_id) {
            return Some(l.to_u128() as usize);
        }

        // Trace through instruction results
        if let Value::Instruction { instruction, position, .. } = &self.dfg[value_id] {
            if let Instruction::Call { func, arguments } = &self.dfg[*instruction] {
                // resize_array(len, data, adjust) -> (u32 length, [T] data)
                // position 0 = length result
                if matches!(&self.dfg[*func], Value::Intrinsic(Intrinsic::ResizeArray)) {
                    if *position == 0 && arguments.len() == 3 {
                        let input_len = self.resolve_length(arguments[0]);
                        let adjust = self.dfg.get_numeric_constant(arguments[2]);
                        if let (Some(len), Some(adj)) = (input_len, adjust) {
                            let adj_i32 = adj.to_i128() as i32;
                            return Some((len as i32 + adj_i32) as usize);
                        }
                    }
                }
            }
        }

        None
    }

    /// Resolve the capacity of a data array by tracing through the instruction chain.
    /// Handles:
    /// - MakeArray: capacity is the number of elements
    /// - ArraySet: same capacity as the input array (trace through recursively)
    fn resolve_data_capacity(&self, value_id: ValueId) -> Option<usize> {
        // MakeArray: capacity is the element count
        if let Some((elements, _typ)) = self.dfg.get_array_constant(value_id) {
            return Some(elements.len());
        }

        // Trace through instructions that preserve capacity
        if let Value::Instruction { instruction, .. } = &self.dfg[value_id] {
            match &self.dfg[*instruction] {
                // ArraySet returns an array of the same capacity
                Instruction::ArraySet { array, .. } => {
                    return self.resolve_data_capacity(*array);
                }
                // resize_array data result (position 1): capacity = resolve input data + adjust
                Instruction::Call { func, arguments }
                    if matches!(&self.dfg[*func], Value::Intrinsic(Intrinsic::ResizeArray)) =>
                {
                    if arguments.len() == 3 {
                        let input_cap = self.resolve_data_capacity(arguments[1]);
                        let adjust = self.dfg.get_numeric_constant(arguments[2]);
                        if let (Some(cap), Some(adj)) = (input_cap, adjust) {
                            let adj_i32 = adj.to_i128() as i32;
                            return Some((cap as i32 + adj_i32) as usize);
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn expand_vector_enumerate_with_write() {
        // Test: closure returns Field, expansion creates ArraySet chain
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            v7 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v9, v10 = call as_vector(v7) -> (u32, [Field])
            v11, v12 = call vector_enumerate(v4, v5, v9, v10, f1, f2, u32 5, u1 0) -> (u32, [Field])
            return v11, v12
        }
        acir(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        acir(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        // The expansion should:
        // 1. Call closure for each index (0, 1, 2) with user_index=5
        // 2. ArraySet each result into vec_b
        // 3. Return (length, final_vec_b)
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v5, v6 = call as_vector(v3) -> (u32, [Field])
            v10 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v11, v12 = call as_vector(v10) -> (u32, [Field])
            v14 = make_array [Field 0, Field 0, Field 0] : [Field]
            v16 = array_get v6, index u32 0 -> Field
            v17 = array_set v14, index u32 0, value v16
            v19 = array_get v6, index u32 1 -> Field
            v20 = array_set v17, index u32 1, value v19
            v22 = array_get v6, index u32 2 -> Field
            v23 = array_set v20, index u32 2, value v22
            return u32 3, v23
        }
        acir(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        acir(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ");
    }

    #[test]
    fn expand_vector_enumerate_reverse() {
        // Test reverse iteration
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            v7 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v9, v10 = call as_vector(v7) -> (u32, [Field])
            v11, v12 = call vector_enumerate(v4, v5, v9, v10, f1, f2, u32 5, u1 1) -> (u32, [Field])
            return v11, v12
        }
        acir(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        acir(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v5, v6 = call as_vector(v3) -> (u32, [Field])
            v10 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v11, v12 = call as_vector(v10) -> (u32, [Field])
            v14 = make_array [Field 0, Field 0, Field 0] : [Field]
            v16 = array_get v6, index u32 2 -> Field
            v17 = array_set v14, index u32 2, value v16
            v19 = array_get v6, index u32 1 -> Field
            v20 = array_set v17, index u32 1, value v19
            v22 = array_get v6, index u32 0 -> Field
            v23 = array_set v20, index u32 0, value v22
            return u32 3, v23
        }
        acir(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        acir(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ");
    }

    #[test]
    fn does_not_expand_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            v7 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v9, v10 = call as_vector(v7) -> (u32, [Field])
            v11, v12 = call vector_enumerate(v4, v5, v9, v10, f1, f2, u32 5, u1 0) -> (u32, [Field])
            return v11, v12
        }
        brillig(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        brillig(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        // Should remain unchanged because it's brillig
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v5, v6 = call as_vector(v3) -> (u32, [Field])
            v10 = make_array [Field 11, Field 21, Field 31] : [Field; 3]
            v11, v12 = call as_vector(v10) -> (u32, [Field])
            v19, v20 = call vector_enumerate(u32 3, v6, u32 3, v12, f1, f2, u32 5, u1 0) -> (u32, [Field])
            return v19, v20
        }
        brillig(inline) fn closure f1 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        brillig(inline) fn closure f2 {
          b0(v0: u32, v1: [Field], v2: [Field], v3: u32, v4: u32):
            v5 = array_get v1, index v3 -> Field
            return v5
        }
        ");
    }
}
