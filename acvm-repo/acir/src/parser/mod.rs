use num_bigint::BigUint;
use num_traits::Num;
use regex::Regex;
use std::collections::BTreeSet;

pub mod brillig_call_parser;

use crate::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
use crate::circuit::{AssertionPayload, Opcode, OpcodeLocation, PublicInputs, opcodes};
use crate::native_types::{Expression, Witness};
use crate::proto::acir::circuit::opcode::{BrilligCall, Call, MemoryInit, MemoryOp};
use crate::proto::acir::circuit::{Circuit, ExpressionWidth};
pub use acir_field;
pub use acir_field::AcirField;
pub use brillig;

pub use super::circuit::black_box_functions::BlackBoxFunc;
pub use super::circuit::opcodes::InvalidInputBitSize;
pub use brillig_call_parser::BrilligCallParser;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    Expr,
    BlackBoxFuncCall,
    BrilligCall,
    CurrentWitnessIndex,
    PrivateParametersIndices,
    PublicParametersIndices,
    ReturnValueIndices,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction<'a> {
    pub instruction_type: InstructionType,
    pub instruction_body: &'a str,
}

fn parse_str_to_field<F: AcirField>(value: &str) -> Result<F, String> {
    // get the sign
    let is_negative = value.trim().starts_with("-");
    let unsigned_value_string =
        if is_negative { value.strip_prefix("-").unwrap().trim() } else { value.trim() };

    let big_num = if let Some(hex) = unsigned_value_string.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(unsigned_value_string, 10)
    };

    big_num.map_err(|_| "could not convert string to field".to_string()).map(|num| {
        if is_negative {
            -F::from_be_bytes_reduce(&num.to_bytes_be())
        } else {
            F::from_be_bytes_reduce(&num.to_bytes_be())
        }
    })
}

pub fn serialize_acir(input: &str) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        match line {
            l if l.starts_with("BLACKBOX::") => {
                if let Some(stripped) = l.strip_prefix("BLACKBOX::").map(|s| s.trim()) {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::BlackBoxFuncCall,
                        instruction_body: stripped,
                    });
                }
            }
            l if l.starts_with("EXPR") => {
                // Strip "EXPR" and any whitespace after it
                if let Some(stripped) = l.strip_prefix("EXPR").map(|s| s.trim()) {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::Expr,
                        instruction_body: stripped,
                    });
                }
            }
            l if l.starts_with("current witness index :") => {
                if let Some(stripped) = l.strip_prefix("current witness index :").map(|s| s.trim())
                {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::CurrentWitnessIndex,
                        instruction_body: stripped,
                    });
                }
            }
            l if l.starts_with("private parameters indices :") => {
                if let Some(stripped) =
                    l.strip_prefix("private parameters indices :").map(|s| s.trim())
                {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::PrivateParametersIndices,
                        instruction_body: stripped,
                    });
                }
            }
            l if l.starts_with("public parameters indices :") => {
                if let Some(stripped) =
                    l.strip_prefix("public parameters indices :").map(|s| s.trim())
                {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::PublicParametersIndices,
                        instruction_body: stripped,
                    });
                }
            }
            l if l.starts_with("return value indices :") => {
                if let Some(stripped) = l.strip_prefix("return value indices :").map(|s| s.trim()) {
                    instructions.push(Instruction {
                        instruction_type: InstructionType::ReturnValueIndices,
                        instruction_body: stripped,
                    });
                }
            }

            _ => {
                continue;
            }
        }
    }
    instructions
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircuitDescription {
    pub current_witness_index: u32,
    pub expression_width: ExpressionWidth,
    pub private_parameters: BTreeSet<Witness>,
    pub public_parameters: PublicInputs,
    pub return_values: PublicInputs,
}

impl CircuitDescription {
    fn new() -> Self {
        CircuitDescription {
            current_witness_index: 0,
            expression_width: ExpressionWidth { value: None },
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(BTreeSet::new()),
            return_values: PublicInputs(BTreeSet::new()),
        }
    }
}

pub fn get_circuit_description(serialized_acir: &Vec<Instruction>) -> CircuitDescription {
    /// the description part of the circuit starts with one of the following options
    /// current witness index: _u32, the largest index of a witness  
    /// private parameters indices: list of witness indices of private parameters
    /// public parameters indices: list of witness indices of private public parameters
    /// return value indices : the witness indices of the values returned by the function
    let mut current_witness_index: u32 = 0;
    let mut private_parameters: BTreeSet<Witness> = BTreeSet::new();
    let mut public_parameters: BTreeSet<Witness> = BTreeSet::new();
    let mut return_values: BTreeSet<Witness> = BTreeSet::new();
    // we match the lines to fill headers up these values
    for instruction in serialized_acir {
        let parse_indices = |s: &str| -> Vec<u32> {
            let elements = s.strip_prefix("[").unwrap().strip_suffix("]").unwrap();
            if elements.is_empty() {
                return vec![];
            }
            elements
                .split(',')
                .map(|s| s.trim().strip_prefix("_").unwrap().parse::<u32>().unwrap())
                .collect()
        };
        match instruction.instruction_type {
            InstructionType::CurrentWitnessIndex => {
                // Extract witness index from "_X" format
                if let Some(index_str) = instruction.instruction_body.strip_prefix('_') {
                    if let Ok(index) = index_str.parse::<u32>() {
                        current_witness_index = index;
                    }
                }
            }
            InstructionType::PrivateParametersIndices => {
                // the private parameter indices is a string of form [_0, _1, _2, ...]
                // we need to split these by comma and cast each to a u32
                let indices = parse_indices(instruction.instruction_body);
                private_parameters.extend(indices.into_iter().map(|i| Witness::from(i)));
            }
            InstructionType::PublicParametersIndices => {
                // same as above
                let indices = parse_indices(instruction.instruction_body);
                public_parameters.extend(indices.into_iter().map(|i| Witness::from(i)));
            }
            InstructionType::ReturnValueIndices => {
                let indices = parse_indices(instruction.instruction_body);
                return_values.extend(indices.into_iter().map(|i| Witness::from(i)));
            }
            _ => continue,
        }
    }

    CircuitDescription {
        current_witness_index: current_witness_index,
        expression_width: ExpressionWidth { value: None },
        private_parameters: private_parameters,
        public_parameters: PublicInputs(public_parameters),
        return_values: PublicInputs(return_values),
    }
}

pub enum ExpressionTerm<F> {
    MulTerm(F, Witness, Witness),
    LinearTerm(F, Witness),
    Constant(F),
}

pub fn parse_black_box_function_call<F: AcirField>(
    instruction: Instruction,
) -> Result<opcodes::BlackBoxFuncCall<F>, String>
where
    F: AcirField,
{
    /// Format is like BLACKBOX::RANGE [(_4, 222)] []
    /// the different types of black box functions are:
    /// - AES128Encrypt
    /// - AND
    /// - XOR
    /// - RANGE
    /// - Blake2s
    /// - Blake3
    /// - EcdsaSecp256k1
    /// - EcdsaSecp256r1
    /// - MultiScalarMul
    /// - EmbeddedCurveAdd
    /// - Keccakf1600
    /// - RecursiveAggregation
    /// - BigIntAdd
    /// - BigIntSub
    /// - BigIntMul
    /// - BigIntDiv
    /// - BigIntFromLeBytes
    /// - BigIntToLeBytes
    /// - Poseidon2Permutation
    /// - SHA256Compression  
    ///
    if instruction.instruction_type != InstructionType::BlackBoxFuncCall {
        return Err(format!("Expected Expr instruction, got {:?}", instruction.instruction_type));
    }
    let expression_body = instruction.instruction_body;
    let mut trimmed = "";
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
            return Ok(BlackBoxFuncCall::RANGE { input: function_input });
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

pub fn parse_arithmetic_expression<F: AcirField>(
    instruction: Instruction,
) -> Result<Expression<F>, String> {
    if instruction.instruction_type != InstructionType::Expr {
        return Err(format!("Expected Expr instruction, got {:?}", instruction.instruction_type));
    }
    let expression_body = instruction.instruction_body;
    /// the expression body is of form [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]
    /// which corresponds to expression 0 - w0 * w2 - w1 * w2 + w_3 = 0
    /// the elements with 3 elements are mul_terms and the elements with 1 element are linear terms   
    let mut mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
    let mut linear_terms: Vec<(F, Witness)> = Vec::new();
    let mut q_c: F = F::zero();
    //first we split the instruction body to a vector of ExpressionTerm values
    let cleaned =
        expression_body.trim().strip_prefix("[").unwrap().strip_suffix("]").unwrap().trim();

    let re = Regex::new(r"\(([^)]+)\)").unwrap();
    // Collect all matches (terms in parentheses)
    let terms: Vec<&str> = re.find_iter(cleaned).map(|m| m.as_str()).collect();

    // Get the constant term (if any) by splitting on whitespace and taking the last term
    let constant = cleaned.split_whitespace().last().unwrap();
    for term in terms {
        let temp: Vec<&str> = term
            .strip_suffix(")")
            .unwrap()
            .strip_prefix("(")
            .unwrap()
            .trim()
            .split(",")
            .into_iter()
            .map(|a| a.trim())
            .collect();
        match temp.len() {
            3 => {
                // this is a mul term of form (constant, witness , witness)
                // we first get the witness indices
                let first_index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                let second_index = temp[2].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                let coeff = parse_str_to_field(temp[0]).unwrap();
                mul_terms.push((coeff, Witness(first_index), Witness(second_index)));
            }
            2 => {
                // this is a linear_combination term of form (constant, witness)
                let index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                let coeff: F = parse_str_to_field(temp[0]).unwrap();
                linear_terms.push((coeff, Witness(index)));
            }
            _ => {
                return Err("the expression has incorrect terms".to_string());
            }
        }
    }

    let q_c = parse_str_to_field(constant).unwrap();
    Ok(Expression { mul_terms: mul_terms, linear_combinations: linear_terms, q_c: q_c })
}

pub fn parse_memory_init(instruction: Instruction) -> Result<MemoryInit, String> {
    todo!()
}

pub fn parse_memory_op(instruction: Instruction) -> Result<MemoryOp, String> {
    todo!()
}

pub fn parse_brillig_call(instruction: Instruction) -> Result<BrilligCall, String> {
    // BRILLIG CALL func id: inputs: [Single/Array/MemoryArray(Expression/Vec<Expression/BlockId> { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })] outputs: [Simple(Witness(10)), Simple(Witness(11))]
    if instruction.instruction_type != InstructionType::BrilligCall {
        return Err(format!(
            "Expected BrilligCall instruction, got {:?}",
            instruction.instruction_type
        ));
    }
    let instruction_body = instruction.instruction_body;
    let re = Regex::new(r"BRILLIG CALL func .: inputs: \[(.*?)\] outputs: \[(.*?)\]").unwrap();
    let captures = re.captures(instruction_body).unwrap();
    let id = captures.get(1).unwrap().as_str();
    let inputs = captures.get(2).unwrap().as_str();
    let outputs = captures.get(3).unwrap().as_str();
    println!("id: {:?}", id);
    println!("inputs: {:?}", inputs);
    println!("outputs: {:?}", outputs);
    todo!()
}

pub fn parse_call(instruction: Instruction) -> Result<Call, String> {
    todo!()
}

// pub fn build_circuit<F>(serialized_acir: Vec<(&str, &str)>) -> Circuit<F> where F: AcirField{
//     // Circuit::empty()
// }

#[cfg(test)]
mod test {
    use super::*;
    use acir::FieldElement;

    #[test]
    fn test_serialize_acir() {
        let acir_string = "func 0
        current witness index : _1
        private parameters indices : [_0]
        public parameters indices : []
        return value indices : [_1]
        EXPR [ (-2, _0) (1, _1) 0 ]";

        let expected = [
            Instruction {
                instruction_type: InstructionType::CurrentWitnessIndex,
                instruction_body: "_1",
            },
            Instruction {
                instruction_type: InstructionType::PrivateParametersIndices,
                instruction_body: "[_0]",
            },
            Instruction {
                instruction_type: InstructionType::PublicParametersIndices,
                instruction_body: "[]",
            },
            Instruction {
                instruction_type: InstructionType::ReturnValueIndices,
                instruction_body: "[_1]",
            },
            Instruction {
                instruction_type: InstructionType::Expr,
                instruction_body: "[ (-2, _0) (1, _1) 0 ]",
            },
        ];
        assert_eq!(serialize_acir(acir_string), expected);
    }

    #[test]
    fn test_serialize_acir_2() {
        let acir_string = "func 0
        current witness index : _3
        private parameters indices : [_0, _1, _2]
        public parameters indices : []
        return value indices : [_3]
        EXPR [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]";

        let expected = [
            Instruction {
                instruction_type: InstructionType::CurrentWitnessIndex,
                instruction_body: "_3",
            },
            Instruction {
                instruction_type: InstructionType::PrivateParametersIndices,
                instruction_body: "[_0, _1, _2]",
            },
            Instruction {
                instruction_type: InstructionType::PublicParametersIndices,
                instruction_body: "[]",
            },
            Instruction {
                instruction_type: InstructionType::ReturnValueIndices,
                instruction_body: "[_3]",
            },
            Instruction {
                instruction_type: InstructionType::Expr,
                instruction_body: "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]",
            },
        ];
        assert_eq!(serialize_acir(acir_string), expected);
    }

    #[test]
    fn test_parse_indices() {
        let parse_indices = |s: &str| -> Vec<u32> {
            s.strip_prefix("[")
                .unwrap()
                .strip_suffix("]")
                .unwrap()
                .split(',')
                .map(|s| s.trim().strip_prefix("_").unwrap().parse::<u32>().unwrap())
                .collect()
        };
        let indices_string_2 = "[_0]";
        let indices_string = "[_0, _1, _2]";
        let indices = parse_indices(indices_string);
        let indices_2 = parse_indices(indices_string_2);
        assert_eq!(indices, vec![0, 1, 2]);
        assert_eq!(indices_2, vec![0]);
    }

    #[test]
    fn test_get_circuit_description() {
        let acir_string = "func 0
current witness index : _1
private parameters indices : [_0]
public parameters indices : []
return value indices : [_1]
        EXPR [ (-2, _0) (1, _1) 0 ]";
        let serialized_acir = serialize_acir(acir_string);
        let circuit_description = get_circuit_description(&serialized_acir);
        assert_eq!(circuit_description.current_witness_index, 1);
        assert_eq!(circuit_description.private_parameters, BTreeSet::from([Witness(0)]));
        assert_eq!(circuit_description.public_parameters, PublicInputs(BTreeSet::new()));
        assert_eq!(circuit_description.return_values, PublicInputs(BTreeSet::from([Witness(1)])));
    }

    #[test]
    fn test_get_circuit_description_2() {
        let acir_string = "func 0
        current witness index : _3
        private parameters indices : [_0, _1, _2]
        public parameters indices : []
        return value indices : [_3]
        EXPR [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]";
        let serialized_acir = serialize_acir(acir_string);
        let circuit_description = get_circuit_description(&serialized_acir);
        assert_eq!(circuit_description.current_witness_index, 3);
        assert_eq!(
            circuit_description.private_parameters,
            BTreeSet::from([Witness(0), Witness(1), Witness(2)])
        );
        assert_eq!(circuit_description.public_parameters, PublicInputs(BTreeSet::new()));
        assert_eq!(circuit_description.return_values, PublicInputs(BTreeSet::from([Witness(3)])));
    }

    #[test]
    fn test_parsing_expression() {
        let expression_body = "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]";
        let cleaned =
            expression_body.trim().strip_prefix("[").unwrap().strip_suffix("]").unwrap().trim();
        let re = Regex::new(r"\(([^)]+)\)").unwrap();
        let terms: Vec<&str> = re.find_iter(cleaned).map(|m| m.as_str()).collect();
        let constant = cleaned.split_whitespace().last().unwrap();

        assert_eq!(terms, ["(-1, _0, _2)", "(-1, _1, _2)", "(1, _3)"]);
        assert_eq!(constant, "0");
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        let arithmetic_expression: Instruction = Instruction {
            instruction_type: InstructionType::Expr,
            instruction_body: "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]",
        };
        let expression =
            parse_arithmetic_expression::<FieldElement>(arithmetic_expression).unwrap();
        assert_eq!(
            expression,
            Expression {
                mul_terms: vec![
                    (-FieldElement::one(), Witness(0), Witness(2)),
                    (-FieldElement::one(), Witness(1), Witness(2))
                ],
                linear_combinations: vec![(FieldElement::one(), Witness(3))],
                q_c: FieldElement::zero()
            }
        );
    }

    #[test]
    fn test_range_regex() {
        let trimmed = "[(_4, 222)] []";

        let re = Regex::new(r"\[?\(_([0-9]+),\s*([0-9]+)\)\]?\s*\[\]").unwrap();
        let captures = re.captures(trimmed).unwrap();
        let witness_index = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let bit_size = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        assert_eq!(witness_index, 4);
        assert_eq!(bit_size, 222);
    }

    #[test]
    fn test_parse_range_blackbox() {
        let range_constraint = "RANGE [(_6, 222)] []";
        let instruction = Instruction {
            instruction_type: InstructionType::BlackBoxFuncCall,
            instruction_body: range_constraint,
        };
        let black_box_func_call =
            parse_black_box_function_call::<FieldElement>(instruction).unwrap();
        assert_eq!(
            black_box_func_call,
            BlackBoxFuncCall::RANGE { input: FunctionInput::witness(Witness(6), 222) }
        );
    }

    #[test]
    fn test_brillig_call_regex() {
        let instruction = Instruction {
            instruction_type: InstructionType::BrilligCall,
            instruction_body: "func 0: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })] outputs: [Simple(Witness(11)), Array([Witness(12), Witness(13), Witness(14)])]",
        };
        let (inputs, outputs, id) =
            BrilligCallParser::serialize_brillig_call(&instruction).unwrap();
        println!("inputs: {:?}", inputs);
        println!("outputs: {:?}", outputs);
        println!("id: {:?}", id);
    }

    #[test]
    fn test_brillig_inputs_parser() {
        let inputs = "Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [(1 , Witness(0) , Witness(1)) , (1 , Witness(2) , Witness(3))], linear_combinations: [], q_c: 4294967296 }), [Array([Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(1))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }])";

        let (single_inputs, array_inputs, memory_array_inputs) =
            BrilligCallParser::parse_brillig_inputs::<FieldElement>(inputs).unwrap();
        println!("single_inputs: {:?}", single_inputs);
        println!("array_inputs: {:?}", array_inputs);
        println!("memory_array_inputs: {:?}", memory_array_inputs);
    }

    #[test]
    fn test_parse_acir() {
        let acir_string = "
        Compiled ACIR for main (unoptimized):
        func 0
        current witness index : _16
        private parameters indices : [_0, _1, _2]
        public parameters indices : []
        return value indices : [_3]
        BRILLIG CALL func 0: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })], outputs: [Simple(Witness(4)), Simple(Witness(5))]
        BLACKBOX::RANGE [(_4, 222)] []
        BLACKBOX::RANGE [(_5, 32)] []
        EXPR [ (1, _0) (1, _1) (-4294967296, _4) (-1, _5) 0 ]
        EXPR [ (-1, _4) (-1, _6) 5096253676302562286669017222071363378443840053029366383258766538131 ]
        BLACKBOX::RANGE [(_6, 222)] []
        BRILLIG CALL func 1: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(-1, Witness(4))], q_c: 5096253676302562286669017222071363378443840053029366383258766538131 })], outputs: [Simple(Witness(7))]
        EXPR [ (-1, _4, _7) (5096253676302562286669017222071363378443840053029366383258766538131, _7) (1, _8) -1 ]
        EXPR [ (-1, _4, _8) (5096253676302562286669017222071363378443840053029366383258766538131, _8) 0 ]
        EXPR [ (-1, _5, _8) (4026531840, _8) (-1, _9) 0 ]
        BLACKBOX::RANGE [(_9, 33)] []
        BRILLIG CALL func 0: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })], outputs: [Simple(Witness(10)), Simple(Witness(11))]
        BLACKBOX::RANGE [(_10, 222)] []
        BLACKBOX::RANGE [(_11, 32)] []
        EXPR [ (1, _2) (-4294967296, _10) (-1, _11) 0 ]
        EXPR [ (-1, _10) (-1, _12) 5096253676302562286669017222071363378443840053029366383258766538131 ]
        BLACKBOX::RANGE [(_12, 222)] []
        BRILLIG CALL func 1: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(-1, Witness(10))], q_c: 5096253676302562286669017222071363378443840053029366383258766538131 })], outputs: [Simple(Witness(13))]
        EXPR [ (-1, _10, _13) (5096253676302562286669017222071363378443840053029366383258766538131, _13) (1, _14) -1 ]
        EXPR [ (-1, _10, _14) (5096253676302562286669017222071363378443840053029366383258766538131, _14) 0 ]
        EXPR [ (-1, _11, _14) (4026531840, _14) (-1, _15) 0 ]
        BLACKBOX::RANGE [(_15, 33)] []
        EXPR [ (1, _5, _11) (-1, _16) 0 ]
        BLACKBOX::RANGE [(_16, 32)] []
        EXPR [ (1, _3) (-1, _16) 0 ]

        unconstrained func 0
        [Const { destination: Direct(10), bit_size: Integer(U32), value: 2 }, Const { destination: Direct(11), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(10), offset_address: Direct(11) }, BinaryFieldOp { destination: Direct(2), op: IntegerDiv, lhs: Direct(0), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Mul, lhs: Direct(2), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Sub, lhs: Direct(0), rhs: Direct(1) }, Mov { destination: Direct(0), source: Direct(2) }, Stop { return_data: HeapVector { pointer: Direct(11), size: Direct(10) } }]
        unconstrained func 1
        [Const { destination: Direct(21), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(20), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(21), offset_address: Direct(20) }, Const { destination: Direct(2), bit_size: Field, value: 0 }, BinaryFieldOp { destination: Direct(3), op: Equals, lhs: Direct(0), rhs: Direct(2) }, JumpIf { condition: Direct(3), location: 8 }, Const { destination: Direct(1), bit_size: Field, value: 1 }, BinaryFieldOp { destination: Direct(0), op: Div, lhs: Direct(1), rhs: Direct(0) }, Stop { return_data: HeapVector { pointer: Direct(20), size: Direct(21) } }]";

        let serialized_acir = serialize_acir(acir_string);
        for instruction in serialized_acir {
            match instruction.instruction_type {
                InstructionType::BlackBoxFuncCall => {
                    let black_box_func_call =
                        parse_black_box_function_call::<FieldElement>(instruction).unwrap();
                    println!("{:?}", black_box_func_call);
                }
                InstructionType::Expr => {
                    let expression =
                        parse_arithmetic_expression::<FieldElement>(instruction).unwrap();
                    println!("{:?}", expression);
                }
                _ => {}
            }
        }
    }
}
