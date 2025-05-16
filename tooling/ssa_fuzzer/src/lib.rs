#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod builder;
pub mod compiler;
pub mod config;
pub mod helpers;
pub mod runner;
pub mod typed_value;

#[cfg(test)]
mod tests {
    use crate::builder::{FuzzerBuilder, InstructionWithTwoArgs};
    use crate::config;
    use crate::runner::{CompareResults, run_and_compare};
    use crate::typed_value::{TypedValue, ValueType};
    use acvm::FieldElement;
    use acvm::acir::native_types::{Witness, WitnessMap};
    use rand::RngCore;

    struct TestHelper {
        acir_builder: FuzzerBuilder,
        brillig_builder: FuzzerBuilder,
    }

    impl TestHelper {
        fn new(types: Vec<ValueType>) -> Self {
            let mut acir_builder = FuzzerBuilder::new_acir();
            let mut brillig_builder = FuzzerBuilder::new_brillig();
            for type_ in types {
                acir_builder.insert_variable(type_.to_ssa_type());
                brillig_builder.insert_variable(type_.to_ssa_type());
            }
            Self { acir_builder, brillig_builder }
        }

        fn insert_instruction_double_arg(
            &mut self,
            instruction: InstructionWithTwoArgs,
            first_arg: TypedValue,
            second_arg: TypedValue,
        ) -> (TypedValue, TypedValue) {
            let acir_return =
                instruction(&mut self.acir_builder, first_arg.clone(), second_arg.clone());
            let brillig_return = instruction(&mut self.brillig_builder, first_arg, second_arg);
            (acir_return, brillig_return)
        }

        fn finalize_function(&mut self, return_value: TypedValue) {
            self.acir_builder.finalize_function(return_value.clone());
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

    fn compare_results(computed_rust: u64, computed_noir: FieldElement) {
        let computed_rust = FieldElement::from(computed_rust);
        assert_eq!(computed_rust, computed_noir, "Noir doesn't match Rust");
    }

    /// Runs the given instruction with the given values and returns the results of the ACIR and Brillig programs
    /// Instruction runned with first and second witness given
    fn run_instruction_double_arg(
        instruction: InstructionWithTwoArgs,
        values: Vec<u64>,
    ) -> FieldElement {
        let lhs = TypedValue::from_value_type(0, &ValueType::U64);
        let rhs = TypedValue::from_value_type(1, &ValueType::U64);
        let mut test_helper = TestHelper::new(vec![ValueType::U64; 7]);
        let witness_map = get_witness_map(values.clone());
        let initial_witness = witness_map;
        let (acir_result, brillig_result) =
            test_helper.insert_instruction_double_arg(instruction, lhs, rhs);
        test_helper.finalize_function(acir_result);
        test_helper.finalize_function(brillig_result);
        let acir_program = test_helper.acir_builder.compile().unwrap();
        let brillig_program = test_helper.brillig_builder.compile().unwrap();
        let result_witness = Witness(config::NUMBER_OF_VARIABLES_INITIAL);
        let compare_results = run_and_compare(
            &acir_program.program,
            &brillig_program.program,
            initial_witness,
            result_witness,
            result_witness,
        );
        // If not agree throw panic, it is not intended to happen in tests
        match compare_results {
            CompareResults::Agree(result) => result,
            CompareResults::Disagree(acir_result, brillig_result) => {
                panic!(
                    "ACIR and Brillig results disagree: ACIR: {}, Brillig: {}, values: {:?}",
                    acir_result,
                    brillig_result,
                    values.clone()
                );
            }
            CompareResults::BothFailed(acir_error, brillig_error) => {
                panic!(
                    "Both ACIR and Brillig failed: ACIR: {}, Brillig: {}, values: {:?}",
                    acir_error,
                    brillig_error,
                    values.clone()
                );
            }
            CompareResults::AcirFailed(acir_error, brillig_result) => {
                panic!(
                    "ACIR failed: ACIR: {}, Brillig: {}, values: {:?}",
                    acir_error,
                    brillig_result,
                    values.clone()
                );
            }
            CompareResults::BrilligFailed(brillig_error, acir_result) => {
                panic!(
                    "Brillig failed: Brillig: {}, ACIR: {}, values: {:?}",
                    brillig_error, acir_result, values
                );
            }
        }
    }

    #[test]
    fn test_add() {
        let mut values = generate_values();
        // to prevent `attempt to add with overflow`
        values[0] %= 12341234;
        let noir_res = run_instruction_double_arg(
            FuzzerBuilder::insert_add_instruction_checked,
            values.clone(),
        );
        compare_results(values[0] + values[1], noir_res);
    }

    #[test]
    fn test_sub() {
        let mut values = generate_values();
        // to prevent `attempt to subtract with overflow`
        if values[0] < values[1] {
            values.swap(0, 1);
        }
        let noir_res = run_instruction_double_arg(
            FuzzerBuilder::insert_sub_instruction_checked,
            values.clone(),
        );
        compare_results(values[0] - values[1], noir_res);
    }

    #[test]
    fn test_mul() {
        let mut values = generate_values();
        // to prevent `attempt to multiply with overflow`
        values[0] %= 12341234;
        values[1] %= 12341234;
        let noir_res = run_instruction_double_arg(
            FuzzerBuilder::insert_mul_instruction_checked,
            values.clone(),
        );
        compare_results(values[0] * values[1], noir_res);
    }

    #[test]
    fn test_div() {
        let mut values = generate_values();
        if values[1] == 0 {
            values[1] = 1;
        }
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_div_instruction, values.clone());
        compare_results(values[0] / values[1], noir_res);
    }

    #[test]
    fn test_mod() {
        let values = generate_values();
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_mod_instruction, values.clone());
        compare_results(values[0] % values[1], noir_res);
    }

    #[test]
    fn test_and() {
        let values = generate_values();
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_and_instruction, values.clone());
        compare_results(values[0] & values[1], noir_res);
    }

    #[test]
    fn test_or() {
        let values = generate_values();
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_or_instruction, values.clone());
        compare_results(values[0] | values[1], noir_res);
    }

    #[test]
    fn test_xor() {
        let values = generate_values();
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_xor_instruction, values.clone());
        compare_results(values[0] ^ values[1], noir_res);
    }
    #[test]
    fn test_shr() {
        let mut values = generate_values();
        values[1] %= 64;
        let noir_res =
            run_instruction_double_arg(FuzzerBuilder::insert_shr_instruction, values.clone());
        compare_results(values[0] >> values[1], noir_res);
    }
}
