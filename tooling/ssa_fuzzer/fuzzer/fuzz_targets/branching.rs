// TODO(sn): refactor this it seems like it is not working 
// do not read it

#![no_main]

use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use ssa_fuzzer::{
    builder::FuzzerBuilder,
    config,
    config::NUMBER_OF_VARIABLES_INITIAL,
    helpers::{id_to_int, u32_to_id_value, u32_to_id_basic_block},
    runner::{run_and_compare, execute_single},
};
use noirc_evaluator::ssa::ir::types::Type;
use acvm::acir::native_types::{Witness, WitnessMap};
use acvm::FieldElement;
use log;
use env_logger;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::Value;
use noirc_driver::{CompiledProgram, CompileError};

// so small to make more eq variables, just to add more booleans
#[derive(Arbitrary, Debug, Clone, Hash)]
enum Instruction {
    Eq {
        lhs: u32,
        rhs: u32,
    },
    Add {
        lhs: u32,
        rhs: u32,
    },
    Sub {
        lhs: u32,
        rhs: u32,
    },
}

#[derive(Arbitrary, Debug, Clone, Hash)]
enum Terminator {
    Return {
        return_value_index: u32,
    },
    Jmp {
        destination_block_index: u32,
    },
    JmpIf {
        condition_index: u32,
        then_destination_block_index: u32,
        else_destination_block_index: u32,
    }
}


// used for boolean logic
#[derive(Arbitrary, Debug, Clone, Hash)]
enum LogicalInstruction {
    And {
        lhs: u32,
        rhs: u32,
    },
    Or {
        lhs: u32,
        rhs: u32,
    },
    Xor {
        lhs: u32,
        rhs: u32,
    },
    Eq {
        lhs: u32,
        rhs: u32,
    },
    Lt {
        lhs: u32,
        rhs: u32,
    },
    Not {
        lhs: u32,
    },
    TerminateWith {
        terminator: Terminator,
        block_index: u32,
    }
}

fn index_presented(index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    acir_witnesses_indeces.contains(&index) && brillig_witnesses_indeces.contains(&index)
}

fn both_indices_presented(first_index: u32, second_index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    index_presented(first_index, acir_witnesses_indeces, brillig_witnesses_indeces) && index_presented(second_index, acir_witnesses_indeces, brillig_witnesses_indeces)
}

fn insert_instruction_with_double_args(
    acir_builder: &mut FuzzerBuilder,
    brillig_builder: &mut FuzzerBuilder,
    lhs: u32, 
    rhs: u32, 
    f: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>, 
    acir_vars: &mut Vec<u32>,
    brillig_vars: &mut Vec<u32>
) {
    if !acir_vars.contains(&lhs) || !acir_vars.contains(&rhs) 
        || !brillig_vars.contains(&lhs) || !brillig_vars.contains(&rhs) {
        return;
    }
    let lhs = u32_to_id_value(lhs);
    let rhs = u32_to_id_value(rhs);
    let acir_result = f(acir_builder, lhs, rhs);
    let brillig_result = f(brillig_builder, lhs, rhs);
    let acir_result = id_to_int(acir_result);
    let brillig_result = id_to_int(brillig_result);
    acir_vars.push(acir_result);
    brillig_vars.push(brillig_result);
}

fn insert_instruction_with_single_arg(
    acir_builder: &mut FuzzerBuilder,
    brillig_builder: &mut FuzzerBuilder,
    arg: u32, 
    f: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>,
    acir_vars: &mut Vec<u32>,
    brillig_vars: &mut Vec<u32>
) {
    if !acir_vars.contains(&arg) || !brillig_vars.contains(&arg) {
        return;
    }
    let arg = u32_to_id_value(arg);
    let acir_result = f(acir_builder, arg);
    let brillig_result = f(brillig_builder, arg);
    let acir_result = id_to_int(acir_result);
    let brillig_result = id_to_int(brillig_result);
    acir_vars.push(acir_result);
    brillig_vars.push(brillig_result);
}

struct FuzzerContext {
    acir_builder: FuzzerBuilder,
    brillig_builder: FuzzerBuilder,
    acir_variables_indices: Vec<u32>,
    brillig_variables_indices: Vec<u32>,
    block_acir_boolean_variables_indices: Vec<u32>,
    block_brillig_boolean_variables_indices: Vec<u32>,
    acir_blocks_indices: Vec<u32>,
    brillig_blocks_indices: Vec<u32>,
    acir_entry_block_index: u32,
    brillig_entry_block_index: u32,
    acir_terminated_blocks_indices: Vec<u32>,
    brillig_terminated_blocks_indices: Vec<u32>,
    // if we are in block, it has other context, we cannot use variable from other blocks
    acir_blocks_local_variables: Vec<Vec<u32>>,
    brillig_blocks_local_variables: Vec<Vec<u32>>,
    acir_current_block_variables_indices: Vec<u32>,
    brillig_current_block_variables_indices: Vec<u32>,
}

impl FuzzerContext {
    fn new(type_: Type) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        acir_builder.insert_variables(type_.clone());
        brillig_builder.insert_variables(type_.clone());
        let acir_entry_block = acir_builder.get_entry_block_index();
        let brillig_entry_block = brillig_builder.get_entry_block_index();
        let acir_variables_indices: Vec<u32> = (0..config::NUMBER_OF_VARIABLES_INITIAL).collect();
        let brillig_variables_indices: Vec<u32> = (0..config::NUMBER_OF_VARIABLES_INITIAL).collect();
        let acir_blocks_local_variables: Vec<Vec<u32>> = vec![];
        let brillig_blocks_local_variables: Vec<Vec<u32>> = vec![];

        Self {
            acir_variables_indices: acir_variables_indices.clone(),
            brillig_variables_indices: brillig_variables_indices.clone(),
            block_acir_boolean_variables_indices: vec![],
            block_brillig_boolean_variables_indices: vec![],
            acir_blocks_indices: vec![],
            brillig_blocks_indices: vec![],
            acir_entry_block_index: acir_entry_block,
            brillig_entry_block_index: brillig_entry_block,
            acir_terminated_blocks_indices: vec![acir_entry_block],
            brillig_terminated_blocks_indices: vec![brillig_entry_block],
            acir_blocks_local_variables,
            brillig_blocks_local_variables,
            acir_current_block_variables_indices: acir_variables_indices.clone(),
            brillig_current_block_variables_indices: brillig_variables_indices.clone(),
            acir_builder,
            brillig_builder,
        }
    }

    fn insert_arithmetic_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add { lhs, rhs } => {
                if !both_indices_presented(lhs, rhs, &mut self.acir_current_block_variables_indices, &mut self.brillig_current_block_variables_indices) {
                    return;
                }
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs, 
                    rhs, 
                    |builder, lhs, rhs| builder.insert_add_instruction(lhs, rhs),
                    &mut self.acir_current_block_variables_indices,
                    &mut self.brillig_current_block_variables_indices
                );
            }
            Instruction::Sub { lhs, rhs } => {
                if !both_indices_presented(lhs, rhs, &mut self.acir_current_block_variables_indices, &mut self.brillig_current_block_variables_indices) {
                    return;
                }
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs, 
                    rhs, 
                    |builder, lhs, rhs| builder.insert_sub_instruction(lhs, rhs),
                    &mut self.acir_current_block_variables_indices,
                    &mut self.brillig_current_block_variables_indices
                );
            }
            Instruction::Eq { lhs, rhs } => {
                if !both_indices_presented(lhs, rhs, &mut self.acir_current_block_variables_indices, &mut self.brillig_current_block_variables_indices) {
                    return;
                }
                let lhs = u32_to_id_value(lhs);
                let rhs = u32_to_id_value(rhs);
                let acir_result = self.acir_builder.insert_eq_instruction(lhs, rhs);
                let brillig_result = self.brillig_builder.insert_eq_instruction(lhs, rhs);
                self.block_acir_boolean_variables_indices.push(id_to_int(acir_result));
                self.block_brillig_boolean_variables_indices.push(id_to_int(brillig_result));
            }
        }
    }

    fn insert_logical_instruction(&mut self, instruction: LogicalInstruction) {
        match instruction {
            LogicalInstruction::And { lhs, rhs } => {
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_and_instruction(lhs, rhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::Or { lhs, rhs } => {
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_or_instruction(lhs, rhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::Xor { lhs, rhs } => {
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_xor_instruction(lhs, rhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::Eq { lhs, rhs } => {
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_eq_instruction(lhs, rhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::Lt { lhs, rhs } => {
                insert_instruction_with_double_args(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_lt_instruction(lhs, rhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::Not { lhs } => {
                if !index_presented(lhs, &mut self.acir_current_block_variables_indices, &mut self.brillig_current_block_variables_indices) {
                    return;
                }
                insert_instruction_with_single_arg(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    lhs, 
                    |builder, lhs| builder.insert_not_instruction(lhs),
                    &mut self.block_acir_boolean_variables_indices,
                    &mut self.block_brillig_boolean_variables_indices
                );
            }
            LogicalInstruction::TerminateWith { terminator, block_index } => {
                self.insert_terminator(terminator, block_index);
            }
        }
    }

    fn insert_terminator(&mut self, terminator: Terminator, block_index: u32) {
        // if we already terminated block
        if index_presented(block_index, &mut self.acir_terminated_blocks_indices, &mut self.brillig_terminated_blocks_indices) {
            return;
        }

        // if block is not created
        if !index_presented(block_index, &mut self.acir_blocks_indices, &mut self.brillig_blocks_indices) {
            return;
        }

        // switch to block and insert terminator
        self.acir_builder.switch_to_block(u32_to_id_basic_block(block_index));
        self.brillig_builder.switch_to_block(u32_to_id_basic_block(block_index));

        match terminator {
            Terminator::Return { return_value_index } => {
                if !index_presented(return_value_index, &mut self.acir_variables_indices, &mut self.brillig_variables_indices) {
                    return;
                }
                let return_value = u32_to_id_value(return_value_index);
                self.acir_builder.insert_return_instruction(return_value);
                self.brillig_builder.insert_return_instruction(return_value);
            }
            Terminator::JmpIf { condition_index, then_destination_block_index, else_destination_block_index } => {
                // logic if or field if
                if !index_presented(condition_index, &mut self.block_acir_boolean_variables_indices, &mut self.block_brillig_boolean_variables_indices) {
                    return;
                }
                if !both_indices_presented(then_destination_block_index, else_destination_block_index, &mut self.acir_blocks_indices, &mut self.brillig_blocks_indices) {
                    return;
                }

                let condition = u32_to_id_value(condition_index);
                let then_destination_block = u32_to_id_basic_block(then_destination_block_index);
                let else_destination_block = u32_to_id_basic_block(else_destination_block_index);
                self.acir_builder.insert_jmpif_instruction(condition, then_destination_block, else_destination_block);
                self.brillig_builder.insert_jmpif_instruction(condition, then_destination_block, else_destination_block);
            }
            _ => {
                return;
            }
        }
    }

    fn store_current_block_variables(&mut self) {
        self.acir_blocks_local_variables.push(self.acir_current_block_variables_indices.clone());
        self.brillig_blocks_local_variables.push(self.brillig_current_block_variables_indices.clone());
    }

    fn create_new_block_and_switch(&mut self) {
        // replace all local for block variables with new local variables
        self.store_current_block_variables();

        let new_acir_block_index = self.acir_builder.insert_block();
        let new_brillig_block_index = self.brillig_builder.insert_block();
        self.acir_blocks_indices.push(new_acir_block_index);
        self.brillig_blocks_indices.push(new_brillig_block_index);
        // and set current block variables to block local variables
        self.acir_current_block_variables_indices = self.acir_variables_indices.clone();
        self.brillig_current_block_variables_indices = self.brillig_variables_indices.clone();

        self.acir_builder.switch_to_block(u32_to_id_basic_block(new_acir_block_index));
        self.brillig_builder.switch_to_block(u32_to_id_basic_block(new_brillig_block_index));
        self.block_acir_boolean_variables_indices = vec![];
        self.block_brillig_boolean_variables_indices = vec![];
    }

    fn finalize_block(&mut self, block_index: u32) {
        if index_presented(block_index, &mut self.acir_terminated_blocks_indices, &mut self.brillig_terminated_blocks_indices) {
            return;
        }
        // finalize block with last local variable
        let acir_block_variables = self.acir_blocks_local_variables.get(block_index as usize).unwrap();
        let brillig_block_variables = self.brillig_blocks_local_variables.get(block_index as usize).unwrap();
        let acir_result_index = *acir_block_variables.last().unwrap();
        let brillig_result_index = *brillig_block_variables.last().unwrap();
        self.acir_builder.switch_to_block(u32_to_id_basic_block(block_index));
        self.brillig_builder.switch_to_block(u32_to_id_basic_block(block_index));
      
        self.acir_builder.insert_return_instruction(u32_to_id_value(acir_result_index));
        self.brillig_builder.insert_return_instruction(u32_to_id_value(brillig_result_index));

        self.acir_terminated_blocks_indices.push(block_index);
        self.brillig_terminated_blocks_indices.push(block_index);
    }

    fn finalize_all_blocks(&mut self) {
        // last block not stored
        self.store_current_block_variables();
        let block_indices: Vec<_> = self.acir_blocks_indices.clone();
        for block_index in 0..block_indices.len() {
            self.finalize_block(block_indices[block_index]);
        }

        self.acir_builder.switch_to_block(u32_to_id_basic_block(self.acir_entry_block_index));
        self.brillig_builder.switch_to_block(u32_to_id_basic_block(self.brillig_entry_block_index));
        //jmp to first block and let it crash

        // I HATE THIS

        let acir_true_val = self.acir_builder.numeric_constant(FieldElement::from(1_u32));
        let brillig_true_val = self.brillig_builder.numeric_constant(FieldElement::from(1_u32));
        let first_acir_block = u32_to_id_basic_block(self.acir_blocks_indices[1]);
        let first_brillig_block = u32_to_id_basic_block(self.brillig_blocks_indices[1]);
        self.acir_builder.insert_jmpif_instruction(acir_true_val, first_acir_block, first_acir_block);
        self.brillig_builder.insert_jmpif_instruction(brillig_true_val, first_brillig_block, first_brillig_block);
    }

    fn get_return_witnesses(&mut self) -> (Witness, Witness) {
        let acir_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
        let brillig_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
        (acir_result_witness, brillig_result_witness)
    }

    fn get_programs(self) -> (Result<CompiledProgram, CompileError>, Result<CompiledProgram, CompileError>) {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}

#[derive(Arbitrary, Debug, Clone, Hash)]
struct Block {
    instructions: [Option<Instruction>; config::MAX_NUMBER_OF_INSTRUCTIONS as usize],
    logical_instructions: [Option<LogicalInstruction>; config::MAX_NUMBER_OF_INSTRUCTIONS as usize],
}
#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    blocks: [Block; config::NUMBER_OF_BLOCKS_INITIAL as usize],
    initial_witness: [String; config::NUMBER_OF_VARIABLES_INITIAL as usize],
}

libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    // Initialize logger once
    let _ = env_logger::try_init();
    let type_ = Type::field();
    let mut witness_map = WitnessMap::new();
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::try_from_str(data.initial_witness.get(i as usize).unwrap());
        match value {
            Some(value) => {
                witness_map.insert(witness, value);
            }
            None => {
                return;
            }
        }
    }

    let initial_witness = witness_map;
    log::debug!("instructions: {:?}", data.blocks.clone());
    log::debug!("initial_witness: {:?}", initial_witness);

    let mut fuzzer_context = FuzzerContext::new(type_.clone());
    // first block is entry block
    for instruction in data.blocks[0].instructions.clone() {
        match instruction {
            Some(instruction) => {
                fuzzer_context.insert_arithmetic_instruction(instruction);
            }
            None => {
                continue;
            }
        }
    }
    for logical_instruction in data.blocks[0].logical_instructions.clone() {
        match logical_instruction {
            Some(instruction) => {
                fuzzer_context.insert_logical_instruction(instruction);
            }
            None => {
                continue;
            }
        }
    }
    for block in data.blocks {
        fuzzer_context.create_new_block_and_switch();
        for instruction in block.instructions {
            match instruction {
                Some(instruction) => {
                    fuzzer_context.insert_arithmetic_instruction(instruction);
                }
                None => {
                    continue;
                }
            }
        }
        for logical_instruction in block.logical_instructions {
            match logical_instruction {
                Some(instruction) => {
                    fuzzer_context.insert_logical_instruction(instruction);
            }
            None => {
                continue;
                }
            }
        }
    }

    fuzzer_context.finalize_all_blocks();
    let (acir_result_witness, brillig_result_witness) = fuzzer_context.get_return_witnesses();
    
    let (acir_program, brillig_program) = fuzzer_context.get_programs();
    let (acir_program, brillig_program) = match (acir_program, brillig_program) {
        (Ok(acir), Ok(brillig)) => (acir, brillig),
        (Err(_), Err(_)) => {
            return;
        }
        (Ok(acir), Err(e)) => {
            let acir_result = execute_single(&acir.program, initial_witness, acir_result_witness);
            match acir_result {
                Ok(result) => {
                    println!("ACIR compiled and successfully executed. Execution result of acir only {:?}", result);
                    panic!("ACIR compiled and successfully executed, 
                    but brillig compilation failed. Execution result of 
                    acir only {:?}. Brillig compilation failed with: {:?}", result, e);
                }
                Err(_e) => {
                    // if acir compiled, but didnt execute and brillig didnt compile, it's ok
                    return;
                }
            }
        }
        (Err(e), Ok(brillig)) => {
            let brillig_result = execute_single(&brillig.program, initial_witness, brillig_result_witness);
            match brillig_result {
                Ok(result) => {
                    println!("Brillig compiled and successfully executed. Execution result of brillig only {:?}", result);
                    panic!("Brillig compiled and successfully executed, 
                    but acir compilation failed. Execution result of 
                    brillig only {:?}. Acir compilation failed with: {:?}", result, e);
                }
                Err(_e) => {
                    // if brillig compiled, but didnt execute and acir didnt compile, it's ok
                    return;
                }
            }
        }
    };

    let (result, acir_result, brillig_result) = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_result_witness, brillig_result_witness);
    log::debug!("result: {:?}", result);
    log::debug!("acir_result: {:?}", acir_result);
    log::debug!("brillig_result: {:?}", brillig_result);

    assert!(result);
});
