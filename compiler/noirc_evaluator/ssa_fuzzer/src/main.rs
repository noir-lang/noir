mod builder;
mod compiler;
mod runner;
mod helpers;
mod config;


use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;
use builder::FuzzerBuilder;
use acvm::{
    acir::native_types::{Witness, WitnessMap},
    FieldElement
};

fn main() {
    let mut brillig_program_builder = FuzzerBuilder::new_brillig();
    brillig_program_builder.insert_variables(Type::signed(8));
    let brillig_result = brillig_program_builder.insert_sub_instruction(Id::new(0), Id::new(1));
    brillig_program_builder.finalize_function(brillig_result);
    let brillig_program = brillig_program_builder.compile().unwrap();
    
    let mut acir_program_builder = FuzzerBuilder::new_acir();
    acir_program_builder.insert_variables(Type::signed(8));
    let acir_result = acir_program_builder.insert_sub_instruction(Id::new(0), Id::new(1));
    acir_program_builder.finalize_function(acir_result);
    let acir_program = acir_program_builder.compile().unwrap();



    let mut initial_witness = WitnessMap::new();
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::from(i);
        initial_witness.insert(witness, value);
    }

    let acir_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
    let brillig_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);

    println!("{:?}", acir_program);
    println!("{:?}", brillig_program);
    let result = runner::run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_result_witness, brillig_result_witness);
    println!("{:?}", result);
}