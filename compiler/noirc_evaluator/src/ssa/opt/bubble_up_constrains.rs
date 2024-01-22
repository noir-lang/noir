use std::collections::HashMap;

use crate::ssa::{
    ir::instruction::{Instruction, InstructionId},
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to go through each instruction and move every `Instruction::Constrain` to immediately
    /// after when all of its inputs are available.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn bubble_up_constrains(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            for block in function.reachable_blocks() {
                let instructions = function.dfg[block].take_instructions();
                let mut filtered_instructions = Vec::with_capacity(instructions.len());

                // Multiple constrains can bubble up to sit under a single instruction. We want to maintain the ordering of these constraints,
                // so we need to keep track of how many constraints are attached to a given instruction.
                // Some assertions don't operate on instruction results, so we use Option so we also track the None case
                let mut inserted_at_instruction: HashMap<Option<InstructionId>, usize> =
                    HashMap::with_capacity(instructions.len());

                let dfg = &function.dfg;
                for instruction in instructions {
                    let (lhs, rhs) = match dfg[instruction] {
                        Instruction::Constrain(lhs, rhs, ..) => (lhs, rhs),
                        _ => {
                            filtered_instructions.push(instruction);
                            continue;
                        }
                    };

                    let last_instruction_that_creates_inputs = filtered_instructions
                        .iter()
                        .rev()
                        .position(|&instruction_id| {
                            let results = dfg.instruction_results(instruction_id).to_vec();
                            results.contains(&lhs) || results.contains(&rhs)
                        })
                        // We iterate through the previous instructions in reverse order so the index is from the
                        // back of the vector
                        .map(|reversed_index| filtered_instructions.len() - reversed_index - 1);

                    let insertion_index = last_instruction_that_creates_inputs
                        .map(|index| {
                            // We want to insert just after the last instruction that creates the inputs
                            index + 1
                        })
                        // If it doesn't depend from the previous instructions, then we insert at the start
                        .unwrap_or_default();

                    let already_inserted_for_this_instruction = inserted_at_instruction
                        .entry(
                            last_instruction_that_creates_inputs
                                .map(|index| filtered_instructions[index]),
                        )
                        .or_default();

                    filtered_instructions.insert(
                        insertion_index + *already_inserted_for_this_instruction,
                        instruction,
                    );

                    *already_inserted_for_this_instruction += 1;
                }

                *function.dfg[block].instructions_mut() = filtered_instructions;
            }
        }
        self
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::RuntimeType,
            instruction::{Binary, BinaryOp, Instruction},
            map::Id,
            types::Type,
        },
    };

    #[test]
    fn check_bubble_up_constrains() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     v2 = add v1, Field 1
        //     constrain v0 == Field 1 'With message'
        //     constrain v2 == Field 3
        //     constrain v0 == Field 1
        //     constrain v1 == Field 2
        //     constrain v1 == Field 2 'With message'
        // }
        //
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let three = builder.field_constant(3u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v1, BinaryOp::Add, one);
        builder.insert_constrain(v0, one, Some("With message".to_string()));
        builder.insert_constrain(v2, three, None);
        builder.insert_constrain(v0, one, None);
        builder.insert_constrain(v1, two, None);
        builder.insert_constrain(v1, two, Some("With message".to_string()));
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: Field):
        //     constrain v0 == Field 1 'With message'
        //     constrain v0 == Field 1
        //     v1 = add v0, Field 1
        //     constrain v1 == Field 2
        //     constrain v1 == Field 2 'With message'
        //     v2 = add v1, Field 1
        //     constrain v2 == Field 3
        // }
        //
        let ssa = ssa.bubble_up_constrains();
        let main = ssa.main();
        let block = &main.dfg[main.entry_block()];
        assert_eq!(block.instructions().len(), 7);

        let expected_instructions = vec![
            Instruction::Constrain(v0, one, Some("With message".to_string())),
            Instruction::Constrain(v0, one, None),
            Instruction::Binary(Binary { lhs: v0, rhs: one, operator: BinaryOp::Add }),
            Instruction::Constrain(v1, two, None),
            Instruction::Constrain(v1, two, Some("With message".to_string())),
            Instruction::Binary(Binary { lhs: v1, rhs: one, operator: BinaryOp::Add }),
            Instruction::Constrain(v2, three, None),
        ];

        for (index, instruction) in block.instructions().iter().enumerate() {
            assert_eq!(&main.dfg[*instruction], &expected_instructions[index]);
        }
    }
}
