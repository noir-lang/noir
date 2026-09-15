//! Tests that a test case survives the round trip through a target's corpus: a
//! fuzz target reads back the bytes its own mutator wrote, so a case encoded by
//! one callback must decode to the same program in the other.
use crate::corpus::CorpusCodec;
use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

const CODECS: [CorpusCodec; 2] = [CorpusCodec::Json, CorpusCodec::MessagePack];

/// One `main` returning `field_0 + field_1`, which executes to 1.
fn addition_case() -> FuzzerData {
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let arg_1_field = NumericArgument { index: 1, numeric_type: NumericType::Field };
    let add_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_0_field, rhs: arg_1_field }],
    };
    let main_function = FunctionData {
        input_types: default_input_types(),
        commands: vec![],
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    FuzzerData {
        instruction_blocks: vec![add_block],
        functions: vec![main_function],
        initial_witness: default_witness(),
    }
}

/// Every codec decodes its own output back to the case that was encoded, so the
/// program a target compiles is the one its mutator described.
#[test]
fn decoded_case_builds_the_encoded_program() {
    for codec in CODECS {
        let data = addition_case();
        let bytes = codec.encode(&data);
        let decoded = codec
            .decode(&bytes)
            .unwrap_or_else(|error| panic!("{codec:?} failed to decode its own output: {error}"));

        assert_eq!(
            decoded.instruction_blocks.len(),
            data.instruction_blocks.len(),
            "{codec:?} lost instruction blocks"
        );
        let output = fuzz_target(decoded, default_runtimes(), FuzzerOptions::default());
        assert!(output.program.is_some(), "{codec:?} decoded to a case that builds no program");
        assert_eq!(output.get_return_witnesses()[0], FieldElement::from(1_u32));
    }
}

/// A mutator handed bytes it cannot read mutates the default case, which
/// describes no instructions and so builds no program.
#[test]
fn undecodable_bytes_mutate_the_default_case() {
    for codec in CODECS {
        let error = codec.decode(b"not a test case").expect_err("expected a decode failure");
        assert!(!error.is_empty(), "{codec:?} reported an empty decode error");

        let default = codec.decode_or_default(b"not a test case");
        assert!(default.instruction_blocks.is_empty());
        let output = fuzz_target(default, default_runtimes(), FuzzerOptions::default());
        assert!(output.program.is_none());
    }
}
