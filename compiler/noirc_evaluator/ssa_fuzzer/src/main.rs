mod builder;
mod compiler;
mod runner;
mod helpers;
mod config;

use std::time::Instant;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;
use builder::FuzzerBuilder;
use acvm::{
    acir::native_types::{Witness, WitnessMap},
    FieldElement
};

fn main() {
    let type_ = Type::unsigned(64);

    let mut dummy_program = FuzzerBuilder::new_acir();
    dummy_program.insert_variables(type_.clone());
    dummy_program.insert_add_instruction(Id::new(0), Id::new(1));
    dummy_program.finalize_function(Id::new(0));
    let dummy_program = dummy_program.compile().unwrap();


    // ACIR program building
    let acir_start = Instant::now();
    let mut acir_program_builder = FuzzerBuilder::new_acir();
    acir_program_builder.insert_variables(type_.clone());
    let acir_result = acir_program_builder.insert_add_instruction(Id::new(0), Id::new(1));
    acir_program_builder.finalize_function(acir_result);
    let acir_program = acir_program_builder.compile().unwrap();
    let acir_duration = acir_start.elapsed();
    println!("ACIR program building took: {:?}", acir_duration);

    // Brillig program building
    let mut brillig_program_builder = FuzzerBuilder::new_brillig();

    brillig_program_builder.insert_variables(type_.clone());
    //let brillig_result = brillig_program_builder.insert_sub_instruction(Id::new(0), Id::new(1));
    brillig_program_builder.finalize_function(Id::new(1));
    let brillig_start = Instant::now();
    let brillig_program = brillig_program_builder.compile().unwrap();
    let brillig_duration = brillig_start.elapsed();
    println!("Brillig program building took: {:?}", brillig_duration);


    // Witness initialization
    let witness_start = Instant::now();
    let mut initial_witness = WitnessMap::new();
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::from(i);
        initial_witness.insert(witness, value);
    }
    let witness_duration = witness_start.elapsed();
    println!("Witness initialization took: {:?}", witness_duration);

    let acir_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
    let brillig_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);

    //println!("{:?}", acir_program);
    //println!("{:?}", brillig_program);
    
    // Program execution and comparison
    let execution_start = Instant::now();
    let result = runner::run_and_compare(&acir_program.program, &brillig_program.program, initial_witness.clone(), acir_result_witness, brillig_result_witness);
    let execution_duration = execution_start.elapsed();
    println!("Program execution and comparison took: {:?}", execution_duration);

    let execution_start = Instant::now();
    let result = runner::run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_result_witness, brillig_result_witness);
    let execution_duration = execution_start.elapsed();
    println!("Program execution and comparison took: {:?}", execution_duration);
    println!("{:?}", result);
}