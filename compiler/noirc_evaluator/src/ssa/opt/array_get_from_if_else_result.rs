use crate::ssa::{
    ir::{function::Function, instruction::Instruction, types::Type, value::Value},
    ssa_gen::Ssa,
};

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
    pub(crate) fn array_get_from_if_else_result_optimization(mut self) -> Self {
        for function in self.functions.values_mut() {
            optimize_array_get_from_if_else_result(function);
        }

        self
    }
}

fn optimize_array_get_from_if_else_result(function: &mut Function) {
    let block = function.entry_block();
    let dfg = &mut function.dfg;
    let instructions = dfg[block].take_instructions();

    for instruction_id in instructions {
        // Only apply this optimization to ArrayGet
        let Instruction::ArrayGet { array, index } = &dfg[instruction_id].clone() else {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        };

        // Don't optimize if the index is a constant (this is optimized later on in a different way)
        if let Value::NumericConstant { .. } = &dfg[dfg.resolve(*index)] {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        }

        // Only if getting an array from a previous instruction
        let Value::Instruction { instruction, .. } = &dfg[dfg.resolve(*array)] else {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        };

        // Only if that previous instruction is an IfElse
        let Instruction::IfElse { then_condition, then_value, else_condition, else_value } =
            &dfg[*instruction]
        else {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        };

        let then_condition = *then_condition;
        let then_value = *then_value;
        let else_condition = *else_condition;
        let else_value = *else_value;

        let then_value_type = dfg.type_of_value(then_value);

        // Only if the IfElse instruction has an array type
        let Type::Array(element_types, _) = then_value_type else {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        };

        let element_types: &Vec<Type> = &element_types;

        // Only if the array isn't of a tuple type (or a composite type)
        if element_types.len() != 1 {
            dfg[block].instructions_mut().push(instruction_id);
            continue;
        }

        let call_stack_id = dfg.get_instruction_call_stack_id(instruction_id);

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
            Some(element_types.clone()),
            call_stack_id,
        );
        let then_result = then_result.first();

        // Then create an instruction like this, for the else branch:
        //
        //     v13 = array_get v3, index v4
        let else_result = dfg.insert_instruction_and_results(
            Instruction::ArrayGet { array: else_value, index: *index },
            block,
            Some(element_types.clone()),
            call_stack_id,
        );
        let else_result = else_result.first();

        // Finally create an IfElse instruction like this:
        //
        //     v14 = if v0 then v12 else if v1 then v13
        let new_result = dfg.insert_instruction_and_results(
            Instruction::IfElse {
                then_condition,
                then_value: then_result,
                else_condition,
                else_value: else_result,
            },
            block,
            None,
            call_stack_id,
        );
        let new_result = new_result.first();

        // And replace the original instruction's value with this final value
        let results = dfg.instruction_results(instruction_id);
        let result = results[0];
        dfg.set_value_from_id(result, new_result);
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::{Binary, Instruction},
            map::Id,
            types::{NumericType, Type},
        },
    };

    #[test]
    fn check_array_get_from_if_else_result_optimization() {
        // acir(inline) fn main f0 {
        //   b0(v0: [Field; 3], v1: [Field; 3], v2: u1, v3: u32):
        //     v4 = not v2
        //     v5 = if v2 then v0 else if v4 then v1
        //     v6 = array_get v5, index v3
        //     (no terminator instruction)
        // }

        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::Array(Arc::new(vec![Type::field()]), 3));
        let v1 = builder.add_parameter(Type::Array(Arc::new(vec![Type::field()]), 3));
        let v2 = builder.add_parameter(Type::bool());
        let v3 = builder.add_parameter(Type::unsigned(32));

        let v4 = builder.insert_not(v2);
        let v5 = builder
            .insert_instruction(
                Instruction::IfElse {
                    then_condition: v2,
                    then_value: v0,
                    else_condition: v4,
                    else_value: v1,
                },
                None,
            )
            .first();
        builder.insert_array_get(v5, v3, Type::field());

        let ssa = builder.finish();
        println!("{ssa}");

        // Expected output:
        // acir(inline) fn main f0 {
        //   b0(v0: [Field; 3], v1: [Field; 3], v2: u1, v3: u32):
        //       v4 = not v2
        //       v5 = if v2 then v0 else if v4 then v1
        //       v7 = array_get v0, index v3
        //       v8 = array_get v1, index v3
        //       v9 = cast v2 as Field
        //       v10 = cast v4 as Field
        //       v11 = mul v9, v7
        //       v12 = mul v10, v8
        //       v13 = add v11, v12
        //       (no terminator instruction)
        //   }
        let ssa = ssa.array_get_from_if_else_result_optimization();
        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        // Let's check only instructions v7..=v13
        let v7 = &main.dfg[instructions[2]];
        assert_eq!(v7, &Instruction::ArrayGet { array: v0, index: v3 });

        let v8 = &main.dfg[instructions[3]];
        assert_eq!(v8, &Instruction::ArrayGet { array: v1, index: v3 });

        let v9 = &main.dfg[instructions[4]];
        assert_eq!(v9, &Instruction::Cast(v2, NumericType::NativeField));

        let v10 = &main.dfg[instructions[5]];
        assert_eq!(v10, &Instruction::Cast(v4, NumericType::NativeField));

        let v11 = &main.dfg[instructions[6]];
        assert_eq!(
            v11,
            &Instruction::Binary(Binary {
                lhs: main.dfg.instruction_results(instructions[4])[0], // v9
                rhs: main.dfg.instruction_results(instructions[2])[0], // v7
                operator: crate::ssa::ir::instruction::BinaryOp::Mul
            })
        );

        let v12 = &main.dfg[instructions[7]];
        assert_eq!(
            v12,
            &Instruction::Binary(Binary {
                lhs: main.dfg.instruction_results(instructions[5])[0], // v10
                rhs: main.dfg.instruction_results(instructions[3])[0], // v8
                operator: crate::ssa::ir::instruction::BinaryOp::Mul
            })
        );

        let v13 = &main.dfg[instructions[8]];
        assert_eq!(
            v13,
            &Instruction::Binary(Binary {
                lhs: main.dfg.instruction_results(instructions[6])[0], // v11
                rhs: main.dfg.instruction_results(instructions[7])[0], // v12
                operator: crate::ssa::ir::instruction::BinaryOp::Add
            })
        );
    }
}
