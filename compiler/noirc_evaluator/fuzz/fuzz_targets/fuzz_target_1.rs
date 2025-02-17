#![no_main]

use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use ssa_fuzzer::builder::FuzzerBuilder;
use ssa_fuzzer::config;
use ssa_fuzzer::config::NUMBER_OF_VARIABLES_INITIAL;
use ssa_fuzzer::helpers::id_to_witness;
use ssa_fuzzer::helpers::id_to_int;
use ssa_fuzzer::helpers::u32_to_id;
use ssa_fuzzer::runner::run_and_compare;
use noirc_evaluator::ssa::ir::types::Type;
use acvm::acir::native_types::Witness;
use acvm::acir::native_types::WitnessMap;
use acvm::FieldElement;
use std::fmt::Debug;
use log;
use env_logger;

#[derive(Arbitrary, Debug, Clone)]
enum Instructions {
    Add {
        lhs: u32,
        rhs: u32,
    },
    Sub {
        lhs: u32,
        rhs: u32,
    },
    Mul {
        lhs: u32,
        rhs: u32,
    },
    Div {
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
}

fn index_presented(index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    acir_witnesses_indeces.contains(&index) && brillig_witnesses_indeces.contains(&index)
}

fn both_indeces_presented(first_index: u32, second_index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    index_presented(first_index, acir_witnesses_indeces, brillig_witnesses_indeces) && index_presented(second_index, acir_witnesses_indeces, brillig_witnesses_indeces)
}

libfuzzer_sys::fuzz_target!(|methods: Vec<Instructions>| {
    // Initialize logger once
    let _ = env_logger::try_init();

    let mut acir_builder = FuzzerBuilder::new_acir();
    let mut brillig_builder = FuzzerBuilder::new_brillig();
    let type_ = Type::unsigned(64);
    acir_builder.insert_variables(type_.clone());
    brillig_builder.insert_variables(type_.clone());

    let mut acir_witnesses_indeces = vec![];
    let mut brillig_witnesses_indeces = vec![];
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        acir_witnesses_indeces.push(i);
        brillig_witnesses_indeces.push(i);
    }

    if let Ok(seed) = std::env::var("RUST_FUZZER_SEED") {
        log::debug!("Current seed: {}", seed);
    }

    let mut initial_witness = WitnessMap::new();
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::from(i);
        initial_witness.insert(witness, value);
    }
    log::debug!("instructions: {:?}", methods.clone());


    for method in methods {
        match method {
            Instructions::Add { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_add_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_add_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Sub { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_sub_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_sub_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Mul { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_mul_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_mul_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Div { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_div_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_div_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Lt { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_lt_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_lt_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Eq { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_eq_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_eq_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::And { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_and_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_and_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Or { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_or_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_or_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            Instructions::Xor { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let lhs_id = u32_to_id(lhs);
                let rhs_id = u32_to_id(rhs);
                let acir_result = acir_builder.insert_xor_instruction(lhs_id, rhs_id);
                let brillig_result = brillig_builder.insert_xor_instruction(lhs_id, rhs_id);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
        }
    }
    let acir_result_index = *acir_witnesses_indeces.last().unwrap();
    let brillig_result_index = *brillig_witnesses_indeces.last().unwrap();
    acir_builder.finalize_function(u32_to_id(acir_result_index));
    brillig_builder.finalize_function(u32_to_id(brillig_result_index));
    let mut acir_result_witness = Witness(acir_result_index);
    let mut brillig_result_witness = Witness(brillig_result_index);

    if acir_witnesses_indeces.len() as u32 != config::NUMBER_OF_VARIABLES_INITIAL {
        acir_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
        brillig_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
    }


    let acir_program = acir_builder.compile().unwrap();
    let brillig_program = brillig_builder.compile().unwrap();
    
    log::debug!("acir_indeces: {:?}", acir_witnesses_indeces);
    log::debug!("brillig_indeces: {:?}", brillig_witnesses_indeces);
    log::debug!("acir_result_witness: {:?}", acir_result_witness);
    log::debug!("brillig_result_witness: {:?}", brillig_result_witness);
    log::debug!("acir_program: {:?}", acir_program.program);
    log::debug!("brillig_program: {:?}", brillig_program.program);

    let (result, acir_result, brillig_result) = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_result_witness, brillig_result_witness);
    log::debug!("result: {:?}", result);
    log::debug!("acir_result: {:?}", acir_result);
    log::debug!("brillig_result: {:?}", brillig_result);

    assert!(result);
});
