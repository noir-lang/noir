use crate::NUMBER_OF_VARIABLES_INITIAL;
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::InstructionBlock;
use crate::options::FuzzerOptions;
use crate::tests::common::{default_runtimes, default_witness};
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use std::sync::Arc;

#[test]
fn rejects_nested_vectors_in_function_signatures() {
    let _ = env_logger::try_init();

    let nested_vector_type = Type::Array(
        Arc::new(vec![Type::Vector(Arc::new(vec![Type::Numeric(NumericType::Field)]))]),
        1,
    );
    let main_func = FunctionData {
        commands: vec![FuzzerFunctionCommand::InsertFunctionCall {
            function_idx: 0,
            args: [0; NUMBER_OF_VARIABLES_INITIAL as usize],
        }],
        input_types: vec![],
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let called_func = FunctionData {
        commands: vec![],
        input_types: vec![nested_vector_type, Type::Numeric(NumericType::Boolean)],
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: vec![InstructionBlock::default()],
        functions: vec![main_func, called_func],
        initial_witness: default_witness(),
    };

    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());

    assert_eq!(
        result.get_compile_error(),
        Some("Nested vectors, i.e. vectors within an array or vector, are not supported")
    );
    assert!(result.get_return_witnesses().is_empty());
}
