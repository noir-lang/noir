use std::collections::{BTreeMap, HashSet};

use acvm::{
    blackbox_solver::StubbedBlackBoxSolver,
    brillig_vm::{MemoryValue, VMStatus, VM},
    FieldElement,
};

use im::Vector;

use crate::{
    brillig::{
        brillig_gen::{brillig_fn::FunctionContext, gen_brillig_for},
        Brillig,
    },
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            function::{Function, FunctionId, RuntimeType},
            instruction::{Instruction, InstructionId},
            types::Type,
            value::{Value, ValueId},
        },
        Ssa,
    },
};

impl Ssa {
    pub(crate) fn inline_const_brillig_calls(mut self, brillig: &Brillig) -> Self {
        // Collect all brillig functions so that later we can find them when processing a call instruction
        let mut brillig_functions: BTreeMap<FunctionId, Function> = BTreeMap::new();
        for (func_id, func) in &self.functions {
            if let RuntimeType::Brillig(..) = func.runtime() {
                let cloned_function = Function::clone_with_id(*func_id, func);
                brillig_functions.insert(*func_id, cloned_function);
            };
        }

        // Keep track of which brillig functions we couldn't completely inline: we'll remove the ones we could.
        let mut brillig_functions_we_could_not_inline = HashSet::new();

        for func in self.functions.values_mut() {
            func.inline_const_brillig_calls(
                brillig,
                &brillig_functions,
                &mut brillig_functions_we_could_not_inline,
            );
        }

        // Remove the brillig functions that are no longer called
        for func_id in brillig_functions.keys() {
            // We never want to remove the main function (it could be `unconstrained` or it
            // could have been turned into brillig if `--force-brillig` was given)
            if self.main_id == *func_id {
                continue;
            }

            if brillig_functions_we_could_not_inline.contains(func_id) {
                continue;
            }

            // We also don't want to remove entry points
            if self.entry_point_to_generated_index.contains_key(func_id) {
                continue;
            }

            self.functions.remove(func_id);
        }

        self
    }
}

/// Result of trying to evaluate an instruction (any instruction) in this pass.
enum EvaluationResult {
    /// Nothing was done because the instruction wasn't a call to a brillig function,
    /// or some arguments to it were not constants.
    NotABrilligCall,
    /// The instruction was a call to a brillig function, but we couldn't evaluate it.
    CannotEvaluate(FunctionId),
    /// The instruction was a call to a brillig function and we were able to evaluate it,
    /// returning evaluation memory values.
    Evaluated(Vec<MemoryValue<FieldElement>>),
}

impl Function {
    pub(crate) fn inline_const_brillig_calls(
        &mut self,
        brillig: &Brillig,
        brillig_functions: &BTreeMap<FunctionId, Function>,
        brillig_functions_we_could_not_inline: &mut HashSet<FunctionId>,
    ) {
        for block_id in self.reachable_blocks() {
            for instruction_id in self.dfg[block_id].take_instructions() {
                let evaluation_result =
                    self.evaluate_const_brillig_call(instruction_id, brillig, brillig_functions);
                match evaluation_result {
                    EvaluationResult::NotABrilligCall => {
                        self.dfg[block_id].instructions_mut().push(instruction_id);
                    }
                    EvaluationResult::CannotEvaluate(func_id) => {
                        self.dfg[block_id].instructions_mut().push(instruction_id);
                        brillig_functions_we_could_not_inline.insert(func_id);
                    }
                    EvaluationResult::Evaluated(memory_values) => {
                        // Replace the instruction results with the constant values we got
                        let result_ids = self.dfg.instruction_results(instruction_id).to_vec();

                        let mut memory_index = 0;
                        for result_id in result_ids {
                            self.replace_result_id_with_memory_value(
                                result_id,
                                block_id,
                                &memory_values,
                                &mut memory_index,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Replaces `result_id` by taking memory values from `memory_values` starting at `memory_index`
    /// depending on the type of the ValueId (it will read multiple memory values if it's an array).
    fn replace_result_id_with_memory_value(
        &mut self,
        result_id: ValueId,
        block_id: BasicBlockId,
        memory_values: &[MemoryValue<FieldElement>],
        memory_index: &mut usize,
    ) {
        let typ = self.dfg.type_of_value(result_id);
        let new_value =
            self.new_value_for_type_and_memory_values(typ, block_id, memory_values, memory_index);
        self.dfg.set_value_from_id(result_id, new_value);
    }

    /// Creates a new value inside this function by reading it from `memory_values` starting at
    /// `memory_index` depending on the given Type: if it's an array multiple values will be read
    /// and a new `make_array` instruction will be created.
    fn new_value_for_type_and_memory_values(
        &mut self,
        typ: Type,
        block_id: BasicBlockId,
        memory_values: &[MemoryValue<FieldElement>],
        memory_index: &mut usize,
    ) -> ValueId {
        match typ {
            Type::Numeric(_) => {
                let memory = memory_values[*memory_index];
                *memory_index += 1;

                let field_value = match memory {
                    MemoryValue::Field(field_value) => field_value,
                    MemoryValue::Integer(u128_value, _) => u128_value.into(),
                };
                self.dfg.make_constant(field_value, typ)
            }
            Type::Array(types, length) => {
                let mut new_array_values = Vector::new();
                for _ in 0..length {
                    for typ in types.iter() {
                        let new_value = self.new_value_for_type_and_memory_values(
                            typ.clone(),
                            block_id,
                            memory_values,
                            memory_index,
                        );
                        new_array_values.push_back(new_value);
                    }
                }

                let instruction = Instruction::MakeArray {
                    elements: new_array_values,
                    typ: Type::Array(types, length),
                };
                let instruction_id = self.dfg.make_instruction(instruction, None);
                self.dfg[block_id].instructions_mut().push(instruction_id);
                *self.dfg.instruction_results(instruction_id).first().unwrap()
            }
            Type::Reference(_) => {
                panic!("Unexpected reference type in brillig function result")
            }
            Type::Slice(_) => {
                panic!("Unexpected slice type in brillig function result")
            }
            Type::Function => {
                panic!("Unexpected function type in brillig function result")
            }
        }
    }

    /// Tries to evaluate an instruction if it's a call that points to a brillig function,
    /// and all its arguments are constant.
    /// We do this by directly executing the function with a brillig VM.
    fn evaluate_const_brillig_call(
        &self,
        instruction_id: InstructionId,
        brillig: &Brillig,
        brillig_functions: &BTreeMap<FunctionId, Function>,
    ) -> EvaluationResult {
        let instruction = &self.dfg[instruction_id];
        let Instruction::Call { func: func_id, arguments } = instruction else {
            return EvaluationResult::NotABrilligCall;
        };

        let func_value = &self.dfg[*func_id];
        let Value::Function(func_id) = func_value else {
            return EvaluationResult::NotABrilligCall;
        };

        let Some(func) = brillig_functions.get(func_id) else {
            return EvaluationResult::NotABrilligCall;
        };

        if !arguments.iter().all(|argument| self.dfg.is_constant(*argument)) {
            return EvaluationResult::CannotEvaluate(*func_id);
        }

        let mut brillig_arguments = Vec::new();
        for argument in arguments {
            let typ = self.dfg.type_of_value(*argument);
            let Some(parameter) = FunctionContext::try_ssa_type_to_parameter(&typ) else {
                return EvaluationResult::CannotEvaluate(*func_id);
            };
            brillig_arguments.push(parameter);
        }

        // Check that return value types are supported by brillig
        for return_id in func.returns().iter() {
            let typ = func.dfg.type_of_value(*return_id);
            if FunctionContext::try_ssa_type_to_parameter(&typ).is_none() {
                return EvaluationResult::CannotEvaluate(*func_id);
            }
        }

        let Ok(generated_brillig) = gen_brillig_for(func, brillig_arguments, brillig) else {
            return EvaluationResult::CannotEvaluate(*func_id);
        };

        let mut calldata = Vec::new();
        for argument in arguments {
            value_id_to_calldata(*argument, &self.dfg, &mut calldata);
        }

        let bytecode = &generated_brillig.byte_code;
        let foreign_call_results = Vec::new();
        let black_box_solver = StubbedBlackBoxSolver;
        let profiling_active = false;
        let mut vm =
            VM::new(calldata, bytecode, foreign_call_results, &black_box_solver, profiling_active);
        let vm_status: VMStatus<_> = vm.process_opcodes();
        let VMStatus::Finished { return_data_offset, return_data_size } = vm_status else {
            return EvaluationResult::CannotEvaluate(*func_id);
        };

        let memory =
            vm.get_memory()[return_data_offset..(return_data_offset + return_data_size)].to_vec();

        EvaluationResult::Evaluated(memory)
    }
}

fn value_id_to_calldata(value_id: ValueId, dfg: &DataFlowGraph, calldata: &mut Vec<FieldElement>) {
    if let Some(value) = dfg.get_numeric_constant(value_id) {
        calldata.push(value);
        return;
    }

    if let Some((values, _type)) = dfg.get_array_constant(value_id) {
        for value in values {
            value_id_to_calldata(value, dfg, calldata);
        }
        return;
    }

    panic!("Expected ValueId to be numeric constant or array constant");
}

#[cfg(test)]
mod test {
    use crate::ssa::opt::assert_normalized_ssa_equals;

    use super::Ssa;

    #[test]
    fn inlines_brillig_call_without_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1() -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0():
                v0 = add Field 2, Field 3
                return v0
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return Field 5
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn inlines_brillig_call_with_two_field_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3) -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field):
                v2 = add v0, v1
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return Field 5
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn inlines_brillig_call_with_two_i32_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(i32 2, i32 3) -> i32
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: i32, v1: i32):
                v2 = add v0, v1
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return i32 5
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn inlines_brillig_call_with_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3, Field 4) -> [Field; 3]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field, v2: Field):
                v3 = make_array [v0, v1, v2] : [Field; 3]
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v3 = make_array [Field 2, Field 3, Field 4] : [Field; 3]
                return v3
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn inlines_brillig_call_with_composite_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, i32 3, Field 4, i32 5) -> [(Field, i32); 2]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: i32, v2: i32, v3: Field):
                v4 = make_array [v0, v1, v2, v3] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v4 = make_array [Field 2, i32 3, Field 4, i32 5] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn inlines_brillig_call_with_array_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = make_array [Field 2, Field 3] : [Field; 2]
                v1 = call f1(v0) -> Field
                return v1
            }

            brillig(inline) fn one f1 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v2 = array_get v0, index u32 0 -> Field
                v4 = array_get v0, index u32 1 -> Field
                v5 = add v2, v4
                dec_rc v0
                return v5
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v2 = make_array [Field 2, Field 3] : [Field; 2]
                return Field 5
            }
            ";
        let ssa = ssa.inline_const_brillig_calls(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }
}
