#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod builder;
pub mod compiler;
pub mod runner;
pub mod typed_value;

#[cfg(test)]
mod tests {
    use crate::builder::{FuzzerBuilder, FuzzerBuilderError, InstructionWithTwoArgs};
    use crate::runner::{CompareResults, run_and_compare};
    use crate::typed_value::{TypedValue, ValueType};
    use acvm::FieldElement;
    use acvm::acir::native_types::{Witness, WitnessMap};
    use noirc_driver::{CompileOptions, CompiledProgram};
    use rand::RngCore;

    use noirc_evaluator::ssa::ir::instruction::BinaryOp;
    use noirc_evaluator::ssa::ir::types::NumericType;

    const NUMBER_OF_VARIABLES_INITIAL: u32 = 7;

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
            self.acir_builder.finalize_function(&return_value);
            self.brillig_builder.finalize_function(&return_value);
        }
    }

    /// Generates a random array with config::NUMBER_OF_VARIABLES_INITIAL elements
    fn generate_values() -> Vec<u64> {
        let mut rng = rand::thread_rng();
        let mut values = Vec::with_capacity(NUMBER_OF_VARIABLES_INITIAL as usize);
        for _ in 0..NUMBER_OF_VARIABLES_INITIAL {
            values.push(rng.next_u64());
        }
        values
    }

    fn get_witness_map(values: Vec<u64>) -> WitnessMap<FieldElement> {
        let mut witness_map = WitnessMap::new();
        for i in 0..NUMBER_OF_VARIABLES_INITIAL {
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
        let acir_program = test_helper.acir_builder.compile(CompileOptions::default()).unwrap();
        let brillig_program =
            test_helper.brillig_builder.compile(CompileOptions::default()).unwrap();
        let result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
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
                    acir_result, brillig_result, &values
                );
            }
            CompareResults::BothFailed(acir_error, brillig_error) => {
                panic!(
                    "Both ACIR and Brillig failed: ACIR: {}, Brillig: {}, values: {:?}",
                    acir_error, brillig_error, &values
                );
            }
            CompareResults::AcirFailed(acir_error, brillig_result) => {
                panic!(
                    "ACIR failed: ACIR: {}, Brillig: {}, values: {:?}",
                    acir_error, brillig_result, &values
                );
            }
            CompareResults::BrilligFailed(brillig_error, acir_result) => {
                panic!(
                    "Brillig failed: Brillig: {}, ACIR: {}, values: {:?}",
                    brillig_error, acir_result, &values
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

    fn check_expected_validation_error(
        compilation_result: Result<CompiledProgram, FuzzerBuilderError>,
        expected_message: &str,
    ) {
        match compilation_result {
            Ok(_) => panic!("Expected an SSA validation failure"),
            Err(FuzzerBuilderError::RuntimeError(error)) => {
                assert!(error.contains(expected_message));
            }
        }
    }

    #[test]
    fn regression_multiplication_without_range_check() {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();

        let field_acir_var = acir_builder.insert_variable(ValueType::Field.to_ssa_type()).value_id;
        let field_brillig_var =
            brillig_builder.insert_variable(ValueType::Field.to_ssa_type()).value_id;

        let truncated_acir = acir_builder.builder.insert_truncate(field_acir_var, 16, 254);
        let truncated_brillig = brillig_builder.builder.insert_truncate(field_brillig_var, 16, 254);

        let field_casted_i16_acir =
            acir_builder.builder.insert_cast(truncated_acir, NumericType::Signed { bit_size: 16 });
        let field_casted_i16_brillig = brillig_builder
            .builder
            .insert_cast(truncated_brillig, NumericType::Signed { bit_size: 16 });

        let casted_pow_2_acir = acir_builder.builder.insert_binary(
            field_casted_i16_acir,
            BinaryOp::Mul { unchecked: false },
            field_casted_i16_acir,
        );
        let casted_pow_2_brillig = brillig_builder.builder.insert_binary(
            field_casted_i16_brillig,
            BinaryOp::Mul { unchecked: false },
            field_casted_i16_brillig,
        );

        let last_var = acir_builder.builder.insert_binary(
            casted_pow_2_acir,
            BinaryOp::Div,
            field_casted_i16_acir,
        );
        let last_var_brillig = brillig_builder.builder.insert_binary(
            casted_pow_2_brillig,
            BinaryOp::Div,
            field_casted_i16_brillig,
        );

        acir_builder.builder.terminate_with_return(vec![last_var]);
        brillig_builder.builder.terminate_with_return(vec![last_var_brillig]);

        let acir_result = acir_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            acir_result,
            "Signed binary operation does not follow overflow pattern",
        );

        let brillig_result = brillig_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            brillig_result,
            "Signed binary operation does not follow overflow pattern",
        );
    }

    #[test]
    fn regression_cast_without_truncate() {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();

        let field_var_acir_id_1 =
            acir_builder.insert_variable(ValueType::Field.to_ssa_type()).value_id;
        let u64_var_acir_id_2 = acir_builder.insert_variable(ValueType::U64.to_ssa_type()).value_id;
        let field_var_brillig_id_1 =
            brillig_builder.insert_variable(ValueType::Field.to_ssa_type()).value_id;
        let u64_var_brillig_id_2 =
            brillig_builder.insert_variable(ValueType::U64.to_ssa_type()).value_id;

        let casted_acir = acir_builder
            .builder
            .insert_cast(field_var_acir_id_1, NumericType::Unsigned { bit_size: 64 });
        let casted_brillig = brillig_builder
            .builder
            .insert_cast(field_var_brillig_id_1, NumericType::Unsigned { bit_size: 64 });

        let mul_acir = acir_builder.builder.insert_binary(
            casted_acir,
            BinaryOp::Mul { unchecked: false },
            u64_var_acir_id_2,
        );
        let mul_brillig = brillig_builder.builder.insert_binary(
            casted_brillig,
            BinaryOp::Mul { unchecked: false },
            u64_var_brillig_id_2,
        );

        acir_builder.builder.terminate_with_return(vec![mul_acir]);
        brillig_builder.builder.terminate_with_return(vec![mul_brillig]);

        let acir_result = acir_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            acir_result,
            "Invalid cast from Field, not preceded by valid truncation or known safe value",
        );
        let brillig_result = brillig_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            brillig_result,
            "Invalid cast from Field, not preceded by valid truncation or known safe value",
        );
    }

    #[test]
    fn regression_signed_sub() {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();

        let i16_acir_var_1 = acir_builder.insert_variable(ValueType::I16.to_ssa_type()).value_id;
        let i16_acir_var_2 = acir_builder.insert_variable(ValueType::I16.to_ssa_type()).value_id;
        let i16_brillig_var_1 =
            brillig_builder.insert_variable(ValueType::I16.to_ssa_type()).value_id;
        let i16_brillig_var_2 =
            brillig_builder.insert_variable(ValueType::I16.to_ssa_type()).value_id;

        let sub_acir = acir_builder.builder.insert_binary(
            i16_acir_var_1,
            BinaryOp::Sub { unchecked: false },
            i16_acir_var_2,
        );
        let sub_brillig = brillig_builder.builder.insert_binary(
            i16_brillig_var_1,
            BinaryOp::Sub { unchecked: false },
            i16_brillig_var_2,
        );

        acir_builder.builder.terminate_with_return(vec![sub_acir]);
        brillig_builder.builder.terminate_with_return(vec![sub_brillig]);

        let acir_result = acir_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            acir_result,
            "Signed binary operation does not follow overflow pattern",
        );
        let brillig_result = brillig_builder.compile(CompileOptions::default());
        check_expected_validation_error(
            brillig_result,
            "Signed binary operation does not follow overflow pattern",
        );
    }
}
