use crate::ssa::{
    ir::{function::Function, instruction::Instruction, map::Id, types::Type, value::ValueId},
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    // Given an original IfElse instruction is this:
    //
    //     v10 = if v0 then v2 else if v1 then v3
    //
    // and a later ArrayGet instruction is this:
    //
    //     v11 = array_get v4, index v4
    //
    // we optimize the latter to this:
    //
    //     v12 = array_get v2, index v4
    //     v13 = array_get v3, index v4
    //     v14 = if v0 then v12 else if v1 then v13
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_get_optimization(mut self) -> Self {
        for function in self.functions.values_mut() {
            Context::default().optimize_array_get(function);
        }

        self
    }
}

#[derive(Default)]
struct Context {
    // Given an IfElse instruction, here we map its result to that instruction.
    // We only capture such values if the IfElse values are arrays.
    result_if_else: HashMap<ValueId, Id<Instruction>>,
}

impl Context {
    fn optimize_array_get(&mut self, function: &mut Function) {
        let block = function.entry_block();
        let dfg = &mut function.dfg;
        let instructions = dfg[block].take_instructions();

        for instruction_id in instructions {
            let instruction = dfg[instruction_id].clone();

            match &instruction {
                Instruction::IfElse { then_value, .. } => {
                    let then_value = *then_value;

                    // Only apply this optimization to IfElse where values are arrays
                    let Type::Array(..) = dfg.type_of_value(then_value) else {
                        continue;
                    };

                    let results = dfg.instruction_results(instruction_id);
                    let result = results[0];
                    self.result_if_else.insert(result, instruction_id);

                    dfg[block].instructions_mut().push(instruction_id);
                }
                Instruction::ArrayGet { array, index } => {
                    // If this array get is for an array that is the result of a previous IfElse...
                    if let Some(if_else) = self.result_if_else.get(array) {
                        if let Instruction::IfElse {
                            then_condition,
                            then_value,
                            else_condition,
                            else_value,
                            ..
                        } = &dfg[*if_else]
                        {
                            let then_condition = *then_condition;
                            let then_value = *then_value;
                            let else_condition = *else_condition;
                            let else_value = *else_value;

                            let then_value_type = dfg.type_of_value(then_value);

                            let Type::Array(element_type, _) = then_value_type else {
                                panic!("ice: expected array type, got {:?}", then_value_type);
                            };
                            let element_type: &Vec<Type> = &element_type;

                            let call_stack = dfg.get_call_stack(instruction_id);

                            // Given the original IfElse instruction is this:
                            //
                            //     v10 = if v0 then v2 else if v1 then v3
                            //
                            // and the ArrayGet instruction is this:
                            //
                            //     v11 = array_get v4, index v4

                            // First create an instruction like this, for the then branch:
                            //
                            //     v12 = array_get v2, index v4
                            let then_result = dfg.insert_instruction_and_results(
                                Instruction::ArrayGet { array: then_value, index: *index },
                                block,
                                Some(element_type.clone()),
                                call_stack.clone(),
                            );
                            let then_result = then_result.first();

                            // Then create an instruction like this, for the else branch:
                            //
                            //     v13 = array_get v3, index v4
                            let else_result = dfg.insert_instruction_and_results(
                                Instruction::ArrayGet { array: else_value, index: *index },
                                block,
                                Some(element_type.clone()),
                                call_stack.clone(),
                            );
                            let else_result = else_result.first();

                            // Finally create an IfElse instruction like this:
                            //
                            //     v14 = if v0 then v12 else if v1 then v13
                            let new_result = dfg.insert_instruction_and_results(
                                Instruction::IfElse {
                                    then_condition: then_condition,
                                    then_value: then_result,
                                    else_condition: else_condition,
                                    else_value: else_result,
                                },
                                block,
                                None,
                                call_stack,
                            );
                            let new_result = new_result.first();

                            // And replace the original instruction's value with this final value
                            let results = dfg.instruction_results(instruction_id);
                            let result = results[0];
                            dfg.set_value_from_id(result, new_result);

                            continue;
                        }
                    }

                    dfg[block].instructions_mut().push(instruction_id);
                }
                _ => {
                    dfg[block].instructions_mut().push(instruction_id);
                }
            }
        }
    }
}
