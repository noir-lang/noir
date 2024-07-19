use std::rc::Rc;

use crate::ssa::{
    ir::{
        dfg::CallStack, function::Function, instruction::Instruction, map::Id, types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_get_optimization(mut self) -> Self {
        for function in self.functions.values_mut() {
            // This should match the check in flatten_cfg
            if let crate::ssa::ir::function::RuntimeType::Brillig = function.runtime() {
                continue;
            }

            Context::default().optimize_array_get(function);
        }

        self
    }
}

#[derive(Default)]
struct Context {
    result_if_else: HashMap<ValueId, Id<Instruction>>,
}

impl Context {
    fn optimize_array_get(&mut self, function: &mut Function) {
        let block = function.entry_block();
        let instructions = function.dfg[block].take_instructions();

        for instruction_id in instructions {
            let instruction = function.dfg[instruction_id].clone();

            match &instruction {
                Instruction::IfElse { then_value, .. } => {
                    let then_value = *then_value;

                    // Only apply this optimization to IfElse where values are arrays
                    let Type::Array(..) = function.dfg.type_of_value(then_value) else {
                        continue;
                    };

                    let results = function.dfg.instruction_results(instruction_id);
                    let result = results[0];
                    self.result_if_else.insert(result, instruction_id);

                    function.dfg[block].instructions_mut().push(instruction_id);
                }
                Instruction::ArrayGet { array, index } => {
                    if let Some(if_else) = self.result_if_else.get(array) {
                        if let Instruction::IfElse {
                            then_condition,
                            then_value,
                            else_condition,
                            else_value,
                            ..
                        } = &function.dfg[*if_else]
                        {
                            let then_condition = *then_condition;
                            let then_value = *then_value;
                            let else_condition = *else_condition;
                            let else_value = *else_value;

                            let then_value_type = function.dfg.type_of_value(then_value);

                            let Type::Array(element_type, _) = then_value_type else {
                                panic!("ice: expected array type, got {:?}", then_value_type);
                            };
                            let element_type: &Vec<Type> = &element_type;

                            let then_result = function.dfg.insert_instruction_and_results(
                                Instruction::ArrayGet { array: then_value, index: *index },
                                block,
                                Some(element_type.clone()),
                                CallStack::new(), // TODO: check callstack
                            );
                            let then_result = then_result.first();

                            let else_result = function.dfg.insert_instruction_and_results(
                                Instruction::ArrayGet { array: else_value, index: *index },
                                block,
                                Some(element_type.clone()),
                                CallStack::new(), // TODO: check callstack
                            );
                            let else_result = else_result.first();

                            let new_result = function.dfg.insert_instruction_and_results(
                                Instruction::IfElse {
                                    then_condition: then_condition,
                                    then_value: then_result,
                                    else_condition: else_condition,
                                    else_value: else_result,
                                },
                                block,
                                None,             // TODO: are these needed?
                                CallStack::new(), // TODO: check callstack
                            );
                            let new_result = new_result.first();

                            let results = function.dfg.instruction_results(instruction_id);
                            let result = results[0];
                            function.dfg.set_value_from_id(result, new_result);

                            continue;
                        }
                    }

                    function.dfg[block].instructions_mut().push(instruction_id);
                }
                _ => {
                    function.dfg[block].instructions_mut().push(instruction_id);
                }
            }
        }
    }
}
