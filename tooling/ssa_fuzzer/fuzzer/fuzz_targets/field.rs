#![no_main]

use acvm::acir::native_types::{Witness, WitnessMap};
use acvm::FieldElement;
use env_logger;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use log;
use noirc_driver::{CompileError, CompiledProgram};
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;
use noirc_evaluator::ssa::ir::value::Value;
use ssa_fuzzer::{
    builder::FuzzerBuilder,
    config,
    config::NUMBER_OF_VARIABLES_INITIAL,
    helpers::{id_to_int, u32_to_id_value},
    runner::{execute_single, run_and_compare},
};

#[derive(Arbitrary, Debug, Clone, Hash)]
enum Instructions {
    Add { lhs: u32, rhs: u32 },
    Sub { lhs: u32, rhs: u32 },
    Mul { lhs: u32, rhs: u32 },
    Div { lhs: u32, rhs: u32 },
    Eq { lhs: u32, rhs: u32 },
    /*Lt {
        lhs: u32,
        rhs: u32,
    },*/
    /*And {
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
    },*/
    Mod { lhs: u32, rhs: u32 },
    Not { lhs: u32 },
    Shl { lhs: u32, rhs: u32 },
    Shr { lhs: u32, rhs: u32 },
    SimpleCast { lhs: u32 },
    BigCastAndBack { lhs: u32, size: u32 },
    ArrayGet { array: u32, index: u32 },
    ArraySet { array: u32, index: u32, value: u32 },
    MakeArray { elements: Vec<u32> },
}

fn index_presented(
    index: u32,
    acir_witnesses_indeces: &mut Vec<u32>,
    brillig_witnesses_indeces: &mut Vec<u32>,
) -> bool {
    acir_witnesses_indeces.contains(&index) && brillig_witnesses_indeces.contains(&index)
}

fn both_indeces_presented(
    first_index: u32,
    second_index: u32,
    acir_witnesses_indeces: &mut Vec<u32>,
    brillig_witnesses_indeces: &mut Vec<u32>,
) -> bool {
    index_presented(first_index, acir_witnesses_indeces, brillig_witnesses_indeces)
        && index_presented(second_index, acir_witnesses_indeces, brillig_witnesses_indeces)
}

struct Array {
    id: Id<Value>,
    length: u32,
}

struct FuzzerContext {
    acir_builder: FuzzerBuilder,
    brillig_builder: FuzzerBuilder,
    acir_witnesses_indeces: Vec<u32>,
    brillig_witnesses_indeces: Vec<u32>,
    acir_arrays: Vec<Array>,
    brillig_arrays: Vec<Array>,
}

impl FuzzerContext {
    fn new(type_: Type) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        acir_builder.insert_variables(type_.clone());
        brillig_builder.insert_variables(type_.clone());
        let mut acir_witnesses_indeces = vec![];
        let mut brillig_witnesses_indeces = vec![];
        for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
            acir_witnesses_indeces.push(i);
            brillig_witnesses_indeces.push(i);
        }
        Self {
            acir_builder,
            brillig_builder,
            acir_witnesses_indeces,
            brillig_witnesses_indeces,
            acir_arrays: vec![],
            brillig_arrays: vec![],
        }
    }

    fn insert_array(&mut self, elements: Vec<u32>) {
        let mut acir_values_ids = vec![];
        let mut brillig_values_ids = vec![];
        for elem in elements {
            if !index_presented(
                elem,
                &mut self.acir_witnesses_indeces,
                &mut self.brillig_witnesses_indeces,
            ) {
                continue;
            }
            acir_values_ids.push(elem);
            brillig_values_ids.push(elem);
        }
        let acir_len = acir_values_ids.len();
        let brillig_len = brillig_values_ids.len();
        let acir_array = self.acir_builder.insert_make_array(acir_values_ids);
        let brillig_array = self.brillig_builder.insert_make_array(brillig_values_ids);
        self.acir_arrays.push(Array { id: acir_array, length: acir_len as u32 });
        self.brillig_arrays.push(Array { id: brillig_array, length: brillig_len as u32 });
    }

    fn insert_array_get(&mut self, array_idx: u32, index: u32) {
        if array_idx >= self.acir_arrays.len() as u32
            || array_idx >= self.brillig_arrays.len() as u32
        {
            return;
        }
        if self.acir_arrays[array_idx as usize].length <= index {
            return;
        }
        if self.brillig_arrays[array_idx as usize].length <= index {
            return;
        }
        let acir_array = self.acir_arrays[array_idx as usize].id;
        let brillig_array = self.brillig_arrays[array_idx as usize].id;
        let acir_result = self.acir_builder.insert_array_get(acir_array, index);
        let brillig_result = self.brillig_builder.insert_array_get(brillig_array, index);
        self.acir_witnesses_indeces.push(id_to_int(acir_result));
        self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
    }

    fn insert_array_set(&mut self, array_idx: u32, index: u32, value: u32) {
        if array_idx >= self.acir_arrays.len() as u32
            || array_idx >= self.brillig_arrays.len() as u32
        {
            return;
        }
        if !index_presented(
            value,
            &mut self.acir_witnesses_indeces,
            &mut self.brillig_witnesses_indeces,
        ) {
            return;
        }
        if self.acir_arrays[array_idx as usize].length <= index {
            return;
        }
        if self.brillig_arrays[array_idx as usize].length <= index {
            return;
        }
        let value = u32_to_id_value(value);
        let acir_array = self.acir_arrays[array_idx as usize].id;
        let brillig_array = self.brillig_arrays[array_idx as usize].id;
        let acir_result = self.acir_builder.insert_array_set(acir_array, index, value);
        let brillig_result = self.brillig_builder.insert_array_set(brillig_array, index, value);
        self.acir_arrays
            .push(Array { id: acir_result, length: self.acir_arrays[array_idx as usize].length });
        self.brillig_arrays.push(Array {
            id: brillig_result,
            length: self.brillig_arrays[array_idx as usize].length,
        });
    }

    fn insert_instruction_with_single_arg(
        &mut self,
        arg: u32,
        f: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>,
    ) {
        if !index_presented(
            arg,
            &mut self.acir_witnesses_indeces,
            &mut self.brillig_witnesses_indeces,
        ) {
            return;
        }
        let arg = u32_to_id_value(arg);
        let acir_result = f(&mut self.acir_builder, arg);
        let brillig_result = f(&mut self.brillig_builder, arg);
        self.acir_witnesses_indeces.push(id_to_int(acir_result));
        self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
    }

    fn insert_instruction_with_double_args(
        &mut self,
        lhs: u32,
        rhs: u32,
        f: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>,
    ) {
        if !both_indeces_presented(
            lhs,
            rhs,
            &mut self.acir_witnesses_indeces,
            &mut self.brillig_witnesses_indeces,
        ) {
            return;
        }
        let lhs = u32_to_id_value(lhs);
        let rhs = u32_to_id_value(rhs);
        let acir_result = f(&mut self.acir_builder, lhs, rhs);
        let brillig_result = f(&mut self.brillig_builder, lhs, rhs);
        self.acir_witnesses_indeces.push(id_to_int(acir_result));
        self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
    }

    fn insert_instruction(&mut self, instruction: Instructions) {
        match instruction {
            Instructions::Add { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_add_instruction(lhs, rhs)
                });
            }
            Instructions::Sub { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_sub_instruction(lhs, rhs)
                });
            }
            Instructions::Mul { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mul_instruction(lhs, rhs)
                });
            }
            Instructions::Div { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_div_instruction(lhs, rhs)
                });
            }
            /*Instructions::Lt { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_lt_instruction(lhs, rhs));
            }*/
            Instructions::Eq { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_eq_instruction(lhs, rhs)
                });
            }
            /*Instructions::And { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_and_instruction(lhs, rhs));
            }
            Instructions::Or { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_or_instruction(lhs, rhs));
            }
            Instructions::Xor { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_xor_instruction(lhs, rhs));
            }*/
            /*
            Instructions::Mod { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_mod_instruction(lhs, rhs));
            }
            Instructions::Not { lhs } => {
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| builder.insert_not_instruction(lhs));
            }*/
            /*Instructions::Shl { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_shl_instruction(lhs, rhs));
            }
            Instructions::Shr { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_shr_instruction(lhs, rhs));
            }*/
            Instructions::SimpleCast { lhs } => {
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| {
                    builder.insert_simple_cast(lhs)
                });
            }
            Instructions::MakeArray { elements } => {
                self.insert_array(elements);
            }
            Instructions::ArrayGet { array, index } => {
                self.insert_array_get(array, index);
            }
            Instructions::ArraySet { array, index, value } => {
                self.insert_array_set(array, index, value);
            }
            Instructions::BigCastAndBack { lhs, size } => {
                if !index_presented(
                    lhs,
                    &mut self.acir_witnesses_indeces,
                    &mut self.brillig_witnesses_indeces,
                ) {
                    return;
                }
                let lhs = u32_to_id_value(lhs);
                let acir_result = self.acir_builder.insert_cast_bigger_and_back(lhs, size);
                let brillig_result = self.brillig_builder.insert_cast_bigger_and_back(lhs, size);
                self.acir_witnesses_indeces.push(id_to_int(acir_result));
                self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            _ => {
                return;
            }
        }
    }

    fn finalize_function(&mut self) {
        let acir_result_index = *self.acir_witnesses_indeces.last().unwrap();
        let brillig_result_index = *self.brillig_witnesses_indeces.last().unwrap();
        self.acir_builder.finalize_function(u32_to_id_value(acir_result_index));
        self.brillig_builder.finalize_function(u32_to_id_value(brillig_result_index));
    }

    fn get_return_witnesses(&mut self) -> (Witness, Witness) {
        let acir_result_index = *self.acir_witnesses_indeces.last().unwrap();
        let brillig_result_index = *self.brillig_witnesses_indeces.last().unwrap();
        let mut acir_result_witness = Witness(acir_result_index);
        let mut brillig_result_witness = Witness(brillig_result_index);

        if self.acir_witnesses_indeces.len() as u32 != config::NUMBER_OF_VARIABLES_INITIAL {
            acir_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
            brillig_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
        }
        (acir_result_witness, brillig_result_witness)
    }

    fn get_programs(
        self,
    ) -> (Result<CompiledProgram, CompileError>, Result<CompiledProgram, CompileError>) {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}

#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    methods: Vec<Instructions>,
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
    log::debug!("instructions: {:?}", data.methods.clone());
    log::debug!("initial_witness: {:?}", initial_witness);

    let mut fuzzer_context = FuzzerContext::new(type_.clone());
    for method in data.methods {
        fuzzer_context.insert_instruction(method);
    }
    fuzzer_context.finalize_function();
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
                    panic!(
                        "ACIR compiled and successfully executed, 
                    but brillig compilation failed. Execution result of 
                    acir only {:?}. Brillig compilation failed with: {:?}",
                        result, e
                    );
                }
                Err(_e) => {
                    // if acir compiled, but didnt execute and brillig didnt compile, it's ok
                    return;
                }
            }
        }
        (Err(e), Ok(brillig)) => {
            let brillig_result =
                execute_single(&brillig.program, initial_witness, brillig_result_witness);
            match brillig_result {
                Ok(result) => {
                    println!("Brillig compiled and successfully executed. Execution result of brillig only {:?}", result);
                    panic!(
                        "Brillig compiled and successfully executed, 
                    but acir compilation failed. Execution result of 
                    brillig only {:?}. Acir compilation failed with: {:?}",
                        result, e
                    );
                }
                Err(_e) => {
                    // if brillig compiled, but didnt execute and acir didnt compile, it's ok
                    return;
                }
            }
        }
    };

    let (result, acir_result, brillig_result) = run_and_compare(
        &acir_program.program,
        &brillig_program.program,
        initial_witness,
        acir_result_witness,
        brillig_result_witness,
    );
    log::debug!("result: {:?}", result);
    log::debug!("acir_result: {:?}", acir_result);
    log::debug!("brillig_result: {:?}", brillig_result);

    assert!(result);
});
