#![allow(dead_code)]
pub mod builder;
pub mod compiler;
pub mod config;
pub mod helpers;
pub mod runner;

#[cfg(test)]
mod tests {
    use crate::builder::FuzzerBuilder;
    use crate::config;
    use noirc_evaluator::ssa::ir::{types::Type, map::Id, value::Value};
    use rand::RngCore;
    use acvm::acir::native_types::{Witness, WitnessMap};
    use acvm::FieldElement;
    use crate::runner::run_and_compare;

    struct TestHelper {
        acir_builder: FuzzerBuilder,
        brillig_builder: FuzzerBuilder,
    }

    impl TestHelper {
        fn new(typ: Type) -> Self {
            let mut acir_builder = FuzzerBuilder::new_acir();
            let mut brillig_builder = FuzzerBuilder::new_brillig();
            acir_builder.insert_variables(typ.clone());
            brillig_builder.insert_variables(typ.clone());
            Self {
                acir_builder,
                brillig_builder,
            }
        }

        fn insert_instruction_double_arg(&mut self, instruction: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>, first_arg: Id<Value>, second_arg: Id<Value>) -> (Id<Value>, Id<Value>) {
            let acir_result = instruction(&mut self.acir_builder, first_arg, second_arg);
            let brillig_result = instruction(&mut self.brillig_builder, first_arg, second_arg); 
            (acir_result, brillig_result)
        }

        fn insert_instruction_single_arg(&mut self, instruction: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>, arg: Id<Value>) -> (Id<Value>, Id<Value>) {
            let acir_result = instruction(&mut self.acir_builder, arg);
            let brillig_result = instruction(&mut self.brillig_builder, arg); 
            (acir_result, brillig_result)
        }

        fn finalize_function(&mut self, return_value: Id<Value>) {
            self.acir_builder.finalize_function(return_value);
            self.brillig_builder.finalize_function(return_value);
        }
    }

    /// Generates a random array with config::NUMBER_OF_VARIABLES_INITIAL elements
    fn generate_values() -> Vec<u64> {
        let mut rng = rand::thread_rng();
        let mut values = Vec::with_capacity(config::NUMBER_OF_VARIABLES_INITIAL as usize);
        for _ in 0..config::NUMBER_OF_VARIABLES_INITIAL {
            values.push(rng.next_u64());
        }
        values
    }

    fn get_witness_map(values: Vec<u64>) -> WitnessMap<FieldElement> {
        let mut witness_map = WitnessMap::new();
        for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
            let witness = Witness(i);
            let value = FieldElement::from(values[i as usize]);
            witness_map.insert(witness, value);
        }
        witness_map
    }

    fn compare_results(computed_rust: u64, computed_acir: FieldElement, computed_brillig: FieldElement) {
        let computed_rust = FieldElement::from(computed_rust);
        assert_eq!(computed_rust, computed_acir);
        assert_eq!(computed_rust, computed_brillig);
    }

    /// Runs the given instruction with the given values and returns the results of the ACIR and Brillig programs
    /// Instruction runned with first and second witness given
    fn run_instruction_double_arg(instruction: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>, values: Vec<u64>) -> (FieldElement, FieldElement) {
        let lhs = Id::new(0);
        let rhs = Id::new(1);
        let mut test_helper = TestHelper::new(Type::unsigned(128));
        let witness_map = get_witness_map(values);
        let initial_witness = witness_map;
        let (acir_result, brillig_result) = test_helper.insert_instruction_double_arg(instruction, lhs, rhs);
        test_helper.finalize_function(acir_result);
        test_helper.finalize_function(brillig_result);
        let acir_program = test_helper.acir_builder.compile().unwrap();
        let brillig_program = test_helper.brillig_builder.compile().unwrap();
        let result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
        let (res, acir_res, brillig_res) = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, result_witness, result_witness);
        assert!(res);
        return (acir_res, brillig_res);
    }

    fn run_instruction_single_arg(instruction: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>, values: Vec<u64>) -> (FieldElement, FieldElement) {
        let arg = Id::new(0);
        let mut test_helper = TestHelper::new(Type::unsigned(128));
        let witness_map = get_witness_map(values);
        let initial_witness = witness_map;
        let (acir_result, brillig_result) = test_helper.insert_instruction_single_arg(instruction, arg);
        test_helper.finalize_function(acir_result);
        test_helper.finalize_function(brillig_result);
        let acir_program = test_helper.acir_builder.compile().unwrap();
        let brillig_program = test_helper.brillig_builder.compile().unwrap();
        let result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
        let (res, acir_res, brillig_res) = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, result_witness, result_witness);
        assert!(res);
        return (acir_res, brillig_res);
    }

    #[test]
    fn test_add() {
        let mut values = generate_values();
        // to prevent `attempt to add with overflow` 
        values[0] %= 12341234;
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_add_instruction_checked, values.clone());
        compare_results(values[0] + values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_sub() {
        let mut values = generate_values();
        // to prevent `attempt to subtract with overflow` 
        if values[0] < values[1] {
            let copy = values[0];
            values[0] = values[1];
            values[1] = copy;
        }
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_sub_instruction_checked, values.clone());
        compare_results(values[0] - values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_mul() {
        let mut values = generate_values();
        // to prevent `attempt to multiply with overflow` 
        values[0] %= 12341234;
        values[1] %= 12341234;
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_mul_instruction_checked, values.clone());
        compare_results(values[0] * values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_div() {
        let values = generate_values();
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_div_instruction, values.clone());
        compare_results(values[0] / values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_mod() {
        let values = generate_values();
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_mod_instruction, values.clone());
        compare_results(values[0] % values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_and() {
        let values = generate_values();
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_and_instruction, values.clone());
        compare_results(values[0] & values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_or() {
        let values = generate_values();
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_or_instruction, values.clone());
        compare_results(values[0] | values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_xor() {
        let values = generate_values();
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_xor_instruction, values.clone());
        compare_results(values[0] ^ values[1], acir_res, brillig_res);
    }

    #[test]
    fn test_shr() {
        let mut values = generate_values();
        values[1] %= 64;
        let (acir_res, brillig_res) = run_instruction_double_arg(FuzzerBuilder::insert_shr_instruction, values.clone());
        compare_results(values[0] >> values[1], acir_res, brillig_res);
    }
}
