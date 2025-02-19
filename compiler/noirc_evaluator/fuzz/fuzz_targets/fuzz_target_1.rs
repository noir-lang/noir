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
use acvm::{FieldElement, AcirField};
use std::fmt::Debug;
use log;
use env_logger;
use fxhash;
use fastrand;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::Value;
use noirc_driver::{CompiledProgram, CompileError};

#[derive(Arbitrary, Debug, Clone, Hash)]
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
    Mod {
        lhs: u32,
        rhs: u32,
    },
    Not {
        lhs: u32,
    },
    Shl {
        lhs: u32,
        rhs: u32,
    },
    Shr {
        lhs: u32,
        rhs: u32,
    },
    SimpleCast {
        lhs: u32,
    },
    BigCastAndBack {
        lhs: u32,
        size: u32,
    },
}

fn index_presented(index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    acir_witnesses_indeces.contains(&index) && brillig_witnesses_indeces.contains(&index)
}

fn both_indeces_presented(first_index: u32, second_index: u32, acir_witnesses_indeces: &mut Vec<u32>, brillig_witnesses_indeces: &mut Vec<u32>) -> bool {
    index_presented(first_index, acir_witnesses_indeces, brillig_witnesses_indeces) && index_presented(second_index, acir_witnesses_indeces, brillig_witnesses_indeces)
}

fn get_witness_map(seed: u64) -> WitnessMap<FieldElement> {
    let mut witness_map = WitnessMap::new();
    let mut rng = fastrand::Rng::with_seed(seed);
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        let value = FieldElement::from(rng.u64(..));
        witness_map.insert(witness, value);
    }
    witness_map
}

struct FuzzerContext {
    acir_builder: FuzzerBuilder,
    brillig_builder: FuzzerBuilder,
    acir_witnesses_indeces: Vec<u32>,
    brillig_witnesses_indeces: Vec<u32>,
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
        }
    }

    fn insert_instruction_with_single_arg(&mut self, arg: u32, f: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>) {
        if !index_presented(arg, &mut self.acir_witnesses_indeces, &mut self.brillig_witnesses_indeces) {
            return;
        }
        let arg = u32_to_id(arg);
        let acir_result = f(&mut self.acir_builder, arg);
        let brillig_result = f(&mut self.brillig_builder, arg);
        self.acir_witnesses_indeces.push(id_to_int(acir_result));
        self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
    }

    fn insert_instruction_with_double_args(&mut self, lhs: u32, rhs: u32, f: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>) {
        if !both_indeces_presented(lhs, rhs, &mut self.acir_witnesses_indeces, &mut self.brillig_witnesses_indeces) {
            return;
        }
        let lhs = u32_to_id(lhs);
        let rhs = u32_to_id(rhs);
        let acir_result = f(&mut self.acir_builder, lhs, rhs);
        let brillig_result = f(&mut self.brillig_builder, lhs, rhs);
        self.acir_witnesses_indeces.push(id_to_int(acir_result));
        self.brillig_witnesses_indeces.push(id_to_int(brillig_result));
    }

    fn insert_instruction(&mut self, instruction: Instructions) {
        match instruction {
            Instructions::Add { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_add_instruction(lhs, rhs));
            }
            Instructions::Sub { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_sub_instruction(lhs, rhs));
            }
            Instructions::Mul { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_mul_instruction(lhs, rhs));
            }
            Instructions::Div { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_div_instruction(lhs, rhs));
            }
            Instructions::Lt { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_lt_instruction(lhs, rhs));
            }
            Instructions::Eq { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_eq_instruction(lhs, rhs));
            }
            Instructions::And { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_and_instruction(lhs, rhs));
            }
            Instructions::Or { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_or_instruction(lhs, rhs));
            }
            Instructions::Xor { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_xor_instruction(lhs, rhs));
            }
            Instructions::Mod { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_mod_instruction(lhs, rhs));
            }
            Instructions::Not { lhs } => {
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| builder.insert_not_instruction(lhs));
            }
            Instructions::Shl { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_shl_instruction(lhs, rhs));
            }
            Instructions::Shr { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| builder.insert_shr_instruction(lhs, rhs));
            }
            Instructions::SimpleCast { lhs } => {
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| builder.insert_simple_cast(lhs));
            }
            /*Instructions::BigCastAndBack { lhs, size } => {
                self.insert_instruction_with_double_args(lhs, size, |builder, lhs, size| builder.insert_cast_bigger_and_back(lhs, size));
            }*/
            _ => {
                return;
            }
        }
    }

    fn finalize_function(&mut self) {
        let acir_result_index = *self.acir_witnesses_indeces.last().unwrap();
        let brillig_result_index = *self.brillig_witnesses_indeces.last().unwrap();
        self.acir_builder.finalize_function(u32_to_id(acir_result_index));
        self.brillig_builder.finalize_function(u32_to_id(brillig_result_index));
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

    fn get_programs(self) -> (Result<CompiledProgram, CompileError>, Result<CompiledProgram, CompileError>) {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}

libfuzzer_sys::fuzz_target!(|methods: Vec<Instructions>| {
    // Initialize logger once
    let _ = env_logger::try_init();
    let seed = fxhash::hash64(&methods);
    let type_ = Type::unsigned(64);
    let initial_witness = get_witness_map(seed);
    log::debug!("instructions: {:?}", methods.clone());
    log::debug!("initial_witness: {:?}", initial_witness);

    let mut fuzzer_context = FuzzerContext::new(type_.clone());
    for method in methods {
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
        (Ok(_), Err(e)) => {
            panic!("ACIR program compiled successfully but Brillig failed with: {:?}", e);
        }
        (Err(e), Ok(_)) => {
            panic!("Brillig program compiled successfully but ACIR failed with: {:?}", e);
        }
    };

    let (result, acir_result, brillig_result) = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_result_witness, brillig_result_witness);
    log::debug!("result: {:?}", result);
    log::debug!("acir_result: {:?}", acir_result);
    log::debug!("brillig_result: {:?}", brillig_result);

    assert!(result);
});
