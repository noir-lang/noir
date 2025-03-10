mod builder;
mod compiler;
mod config;
mod helpers;
mod runner;

use acvm::{
    FieldElement,
    acir::native_types::{Witness, WitnessMap},
};
use builder::FuzzerBuilder;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;

fn main() {
    let type_ = Type::unsigned(128);

    let mut acir_program_builder = FuzzerBuilder::new_acir();
    acir_program_builder.insert_variables(type_.clone());
    let acir_result = acir_program_builder.insert_mul_instruction(Id::new(0), Id::new(0));
    acir_program_builder.finalize_function(acir_result);
    let acir_program = acir_program_builder.compile().unwrap();

    let mut brillig_program_builder = FuzzerBuilder::new_brillig();

    brillig_program_builder.insert_variables(type_.clone());
    let brillig_result = brillig_program_builder.insert_mul_instruction(Id::new(0), Id::new(0));
    brillig_program_builder.finalize_function(brillig_result);
    let brillig_program = brillig_program_builder.compile().unwrap();

    let mut initial_witness = WitnessMap::new();
    let arr: [u64; 7] = [
        3170535237180456974,
        18446742978502394025,
        3946147232088063,
        3170534760439087360,
        169,
        0,
        0,
    ];
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::from(arr[i as usize]);
        initial_witness.insert(witness, value);
    }

    let acir_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
    let brillig_result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);

    println!("{:?}", acir_program);
    //println!("{:?}", brillig_program);
    let result = runner::run_and_compare(
        &acir_program.program,
        &brillig_program.program,
        initial_witness,
        acir_result_witness,
        brillig_result_witness,
    );
    println!("{:?}", result);
}
