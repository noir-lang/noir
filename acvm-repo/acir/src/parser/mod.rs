use regex::Regex;
use std::collections::BTreeSet;

mod arithmetic_parser;
mod black_box_parser;
mod brillig_call_parser;
mod utils;

use crate::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
use crate::circuit::{Opcode, PublicInputs, opcodes};
use crate::native_types::{Expression, Witness};
use crate::proto::acir::circuit::opcode::{BrilligCall, Call, MemoryInit, MemoryOp};
use crate::proto::acir::circuit::{Circuit, ExpressionWidth};
use acir_field::AcirField;

use super::circuit::black_box_functions::BlackBoxFunc;
use super::circuit::opcodes::InvalidInputBitSize;
use arithmetic_parser::ArithmeticParser;
use black_box_parser::BlackBoxParser;
use brillig_call_parser::BrilligCallParser;
use utils::parse_str_to_field;

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

pub struct AcirParser {}

impl AcirParser {
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
                    if let Some(stripped) =
                        l.strip_prefix("current witness index :").map(|s| s.trim())
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
                    if let Some(stripped) =
                        l.strip_prefix("return value indices :").map(|s| s.trim())
                    {
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

    fn get_circuit_description(serialized_acir: &Vec<Instruction>) -> CircuitDescription {
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
}

#[derive(Debug, Clone, PartialEq)]
struct CircuitDescription {
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

pub enum ExpressionTerm<F> {
    MulTerm(F, Witness, Witness),
    LinearTerm(F, Witness),
    Constant(F),
}

pub fn parse_memory_init(instruction: Instruction) -> Result<MemoryInit, String> {
    todo!()
}

pub fn parse_memory_op(instruction: Instruction) -> Result<MemoryOp, String> {
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
        assert_eq!(AcirParser::serialize_acir(acir_string), expected);
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
        assert_eq!(AcirParser::serialize_acir(acir_string), expected);
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
        let serialized_acir = AcirParser::serialize_acir(acir_string);
        let circuit_description = AcirParser::get_circuit_description(&serialized_acir);
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
        let serialized_acir = AcirParser::serialize_acir(acir_string);
        let circuit_description = AcirParser::get_circuit_description(&serialized_acir);
        assert_eq!(circuit_description.current_witness_index, 3);
        assert_eq!(
            circuit_description.private_parameters,
            BTreeSet::from([Witness(0), Witness(1), Witness(2)])
        );
        assert_eq!(circuit_description.public_parameters, PublicInputs(BTreeSet::new()));
        assert_eq!(circuit_description.return_values, PublicInputs(BTreeSet::from([Witness(3)])));
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

        let serialized_acir = AcirParser::serialize_acir(acir_string);
        for instruction in serialized_acir {
            match instruction.instruction_type {
                InstructionType::BlackBoxFuncCall => {
                    let black_box_func_call =
                        BlackBoxParser::parse_black_box_function_call::<FieldElement>(instruction)
                            .unwrap();
                    println!("{:?}", black_box_func_call);
                }
                InstructionType::Expr => {
                    let expression =
                        ArithmeticParser::parse_arithmetic_instruction::<FieldElement>(instruction)
                            .unwrap();
                    println!("{:?}", expression);
                }
                _ => {}
            }
        }
    }
}
