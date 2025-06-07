use regex::Regex;

use crate::acir_field::AcirField;
use crate::parser::{Instruction, InstructionType};
use crate::{
    circuit::{
        black_box_functions::BlackBoxFunc,
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode},
    },
    native_types::Witness,
};

pub struct BlackBoxParser {}

impl BlackBoxParser {
    pub fn parse_black_box_function_call<F: AcirField>(
        instruction: Instruction,
    ) -> Result<Opcode<F>, String>
    where
        F: AcirField,
    {
        // Format is like BLACKBOX::RANGE [(_4, 222)] []
        // the different types of black box functions are:
        // - AES128Encrypt
        // - AND
        // - XOR
        // - RANGE
        // - Blake2s
        // - Blake3
        // - EcdsaSecp256k1
        // - EcdsaSecp256r1
        // - MultiScalarMul
        // - EmbeddedCurveAdd
        // - Keccakf1600
        // - RecursiveAggregation
        // - BigIntAdd
        // - BigIntSub
        // - BigIntMul
        // - BigIntDiv
        // - BigIntFromLeBytes
        // - BigIntToLeBytes
        // - Poseidon2Permutation
        // - SHA256Compression
        if instruction.instruction_type != InstructionType::BlackBoxFuncCall {
            return Err(format!(
                "Expected Expr instruction, got {:?}",
                instruction.instruction_type
            ));
        }
        let expression_body = instruction.instruction_body;
        let trimmed;
        // get the black box function type
        let black_box_type = match expression_body {
            s if s.trim().starts_with("AES128Encrypt") => {
                trimmed = s.trim().strip_prefix("AES128Encrypt").unwrap().trim();
                BlackBoxFunc::AES128Encrypt
            }
            s if s.trim().starts_with("AND") => {
                trimmed = s.trim().strip_prefix("AND").unwrap().trim();
                BlackBoxFunc::AND
            }
            s if s.trim().starts_with("XOR") => {
                trimmed = s.trim().strip_prefix("XOR").unwrap().trim();
                BlackBoxFunc::XOR
            }
            s if s.trim().starts_with("RANGE") => {
                trimmed = s.trim().strip_prefix("RANGE").unwrap().trim();
                BlackBoxFunc::RANGE
            }
            s if s.trim().starts_with("Blake2s") => {
                trimmed = s.trim().strip_prefix("Blake2s").unwrap().trim();
                BlackBoxFunc::Blake2s
            }
            s if s.trim().starts_with("Blake3") => {
                trimmed = s.trim().strip_prefix("Blake3").unwrap().trim();
                BlackBoxFunc::Blake3
            }
            s if s.trim().starts_with("EcdsaSecp256k1") => {
                trimmed = s.trim().strip_prefix("EcdsaSecp256k1").unwrap().trim();
                BlackBoxFunc::EcdsaSecp256k1
            }
            s if s.trim().starts_with("EcdsaSecp256r1") => {
                trimmed = s.trim().strip_prefix("EcdsaSecp256r1").unwrap().trim();
                BlackBoxFunc::EcdsaSecp256r1
            }
            s if s.trim().starts_with("MultiScalarMul") => {
                trimmed = s.trim().strip_prefix("MultiScalarMul").unwrap().trim();
                BlackBoxFunc::MultiScalarMul
            }
            s if s.trim().starts_with("EmbeddedCurveAdd") => {
                trimmed = s.trim().strip_prefix("EmbeddedCurveAdd").unwrap().trim();
                BlackBoxFunc::EmbeddedCurveAdd
            }
            s if s.trim().starts_with("Keccakf1600") => {
                trimmed = s.trim().strip_prefix("Keccakf1600").unwrap().trim();
                BlackBoxFunc::Keccakf1600
            }
            s if s.trim().starts_with("RecursiveAggregation") => {
                trimmed = s.trim().strip_prefix("RecursiveAggregation").unwrap().trim();
                BlackBoxFunc::RecursiveAggregation
            }
            s if s.trim().starts_with("BigIntAdd") => {
                trimmed = s.trim().strip_prefix("BigIntAdd").unwrap().trim();
                BlackBoxFunc::BigIntAdd
            }
            s if s.trim().starts_with("BigIntSub") => {
                trimmed = s.trim().strip_prefix("BigIntSub").unwrap().trim();
                BlackBoxFunc::BigIntSub
            }
            s if s.trim().starts_with("BigIntMul") => {
                trimmed = s.trim().strip_prefix("BigIntMul").unwrap().trim();
                BlackBoxFunc::BigIntMul
            }
            s if s.trim().starts_with("BigIntDiv") => {
                trimmed = s.trim().strip_prefix("BigIntDiv").unwrap().trim();
                BlackBoxFunc::BigIntDiv
            }
            s if s.trim().starts_with("BigIntFromLeBytes") => {
                trimmed = s.trim().strip_prefix("BigIntFromLeBytes").unwrap().trim();
                BlackBoxFunc::BigIntFromLeBytes
            }
            s if s.trim().starts_with("BigIntToLeBytes") => {
                trimmed = s.trim().strip_prefix("BigIntToLeBytes").unwrap().trim();
                BlackBoxFunc::BigIntToLeBytes
            }
            s if s.trim().starts_with("Poseidon2Permutation") => {
                trimmed = s.trim().strip_prefix("Poseidon2Permutation").unwrap().trim();
                BlackBoxFunc::Poseidon2Permutation
            }
            s if s.trim().starts_with("Poseidon2HashCompression") => {
                trimmed = s.trim().strip_prefix("Poseidon2HashCompression").unwrap().trim();
                BlackBoxFunc::Sha256Compression
            }
            _ => return Err(format!("Unknown black box function type in: {}", expression_body)),
        };

        match black_box_type {
            BlackBoxFunc::RANGE => {
                // the format is like BLACKBOX::RANGE [(_4, 222)] []
                let re = Regex::new(r"\[?\(_([0-9]+),\s*([0-9]+)\)\]?\s*\[\]").unwrap();
                let captures = re.captures(trimmed).unwrap();
                let witness_index = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                let bit_size = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                // now we build a FunctionInput struct out of the witness index and bit size
                let function_input: FunctionInput<F> =
                    FunctionInput::witness(Witness(witness_index), bit_size);
                let call = BlackBoxFuncCall::RANGE { input: function_input };

                return Ok(Opcode::BlackBoxFuncCall(call));
            }
            _ => return Err(format!("Unknown black box function type in: {}", expression_body)),
        }

        //     }
        //     BlackBoxFunc::AES128Encrypt => {
        //         // the inputs of the AES128Encrypt are the following:
        //         // input: FunctionInput<F>
        //         // iv: Box<[FunctionInput<F>; 16]>
        //         // key: Box<[FunctionInput<F>; 16]>
        //         // outputs: Vec<Witness>
        //         return Err(format!("AES128Encrypt is not supported yet"));

        //     }
        //     _ => return Err(format!("Unknown black box function type in: {}", expression_body)),
        // }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::acir_field::FieldElement;

    #[test]
    fn test_parse_range_blackbox() {
        let range_constraint = "RANGE [(_6, 222)] []";
        let instruction = Instruction {
            instruction_type: InstructionType::BlackBoxFuncCall,
            instruction_body: range_constraint,
        };
        let black_box_func_call =
            BlackBoxParser::parse_black_box_function_call::<FieldElement>(instruction).unwrap();
        assert_eq!(
            black_box_func_call,
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(6), 222)
            })
        );
    }
}
