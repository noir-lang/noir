use libfuzzer_sys::arbitrary::Arbitrary;
use crate::builder::FuzzerBuilder;
use crate::config;
use crate::helpers::id_to_witness;
use crate::helpers::id_to_int;
use crate::runner::run_and_compare;
use noirc_evaluator::ssa::ir::types::Type;
use acvm::acir::native_types::Witness;
#[derive(Arbitrary, Debug)]
enum AllocatorMethod {
    Add {
        lhs: usize,
        rhs: usize,
    },
    Sub {
        lhs: usize,
        rhs: usize,
    },
}

fn index_presented(index: usize, acir_witnesses_indeces: &mut Vec<usize>, brillig_witnesses_indeces: &mut Vec<usize>) -> bool {
    acir_witnesses_indeces.contains(&index) && brillig_witnesses_indeces.contains(&index)
}

fn both_indeces_presented(first_index: usize, second_index: usize, acir_witnesses_indeces: &mut Vec<usize>, brillig_witnesses_indeces: &mut Vec<usize>) -> bool {
    index_presented(first_index, acir_witnesses_indeces, brillig_witnesses_indeces) && index_presented(second_index, acir_witnesses_indeces, brillig_witnesses_indeces)
}

libfuzzer_sys::fuzz_target!(|methods: Vec<AllocatorMethod>| {
    let mut acir_builder = FuzzerBuilder::new_acir();
    let mut brillig_builder = FuzzerBuilder::new_brillig();
    let type_ = Type::unsigned(16);
    acir_builder.insert_variables(type_);
    brillig_builder.insert_variables(type_);

    let mut acir_witnesses_indeces = vec![];
    let mut brillig_witnesses_indeces = vec![];
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        acir_witnesses_indeces.push(i);
        brillig_witnesses_indeces.push(i);
    }

    for method in methods {
        match method {
            AllocatorMethod::Add { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                let acir_result = acir_builder.insert_add_instruction(lhs, rhs);
                let brillig_result = brillig_builder.insert_add_instruction(lhs, rhs);
                acir_witnesses_indeces.push(id_to_int(acir_result));
                brillig_witnesses_indeces.push(id_to_int(brillig_result));
            }
            AllocatorMethod::Sub { lhs, rhs } => {
                if !both_indeces_presented(lhs, rhs, &mut acir_witnesses_indeces, &mut brillig_witnesses_indeces) {
                    continue;
                }
                acir_builder.insert_sub_instruction(lhs, rhs);
                brillig_builder.insert_sub_instruction(lhs, rhs);
            }
        }
    }

    let acir_program = acir_builder.compile().unwrap();
    let brillig_program = brillig_builder.compile().unwrap();
    let acir_result_index = Witness(*acir_witnesses_indeces.last().unwrap());
    let brillig_result_index = Witness(*brillig_witnesses_indeces.last().unwrap());
    assert!(run_and_compare(&acir_program, &brillig_program, acir_result_index, brillig_result_index));
});
