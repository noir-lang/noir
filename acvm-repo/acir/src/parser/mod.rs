use std::collections::BTreeSet;
use num_bigint::BigUint;
use regex::Regex;
use num_traits::Num;

pub use acir_field;
pub use acir_field::{AcirField};
pub use brillig;
use crate::circuit::{opcodes, AssertionPayload, Opcode, OpcodeLocation, PublicInputs};
use crate::native_types::{Expression, Witness};
use crate::proto::acir::circuit::{Circuit, ExpressionWidth};

pub use super::circuit::black_box_functions::BlackBoxFunc;
pub use super::circuit::opcodes::InvalidInputBitSize;


#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType{
    Expr, 
    CurrentWitnessIndex,
    PrivateParametersIndices,
    PublicParametersIndices,
    ReturnValueIndices,
    Other, 
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction<'a>{
    pub instruction_type: InstructionType,
    pub instruction_body: &'a str, 
}



fn parse_str_to_field<F: AcirField>(value: &str) -> Result<F, String> {
    // get the sign 
    let is_negative = value.trim().starts_with("-"); 
    let unsigned_value_string = if is_negative{
        value.strip_prefix("-").unwrap().trim()
    }else{ 
        value.trim()
    };

    let big_num = if let Some(hex) = unsigned_value_string.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(unsigned_value_string, 10)
    };
    println!("is_negative:{:?}", is_negative); 
    println!("big_num:{:?}", big_num); 
    big_num.map_err(|_| "could not convert string to field".to_string())
        .map(|num| if is_negative {
            - F::from_be_bytes_reduce(&num.to_bytes_be())}
            else{
            F::from_be_bytes_reduce(&num.to_bytes_be())
        })
}


pub fn serialize_acir(input: &str) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    for line in input.lines() {
        let line = line.trim(); 
        match line {
            l if l.starts_with("EXPR") => {
                // Strip "EXPR" and any whitespace after it
                if let Some(stripped) = l.strip_prefix("EXPR").map(|s| s.trim()) {
                    instructions.push(Instruction{instruction_type: InstructionType::Expr, instruction_body: stripped});
                }
            },
            l if l.starts_with("current witness index :") => {
                if let Some(stripped) = l.strip_prefix("current witness index :").map(|s| s.trim()) {
                    instructions.push(Instruction{instruction_type: InstructionType::CurrentWitnessIndex, instruction_body: stripped});
                }
            },
            l if l.starts_with("private parameters indices :") => {
                if let Some(stripped) = l.strip_prefix("private parameters indices :").map(|s| s.trim()) {
                    instructions.push(Instruction{instruction_type: InstructionType::PrivateParametersIndices, instruction_body: stripped});
                }
            },
            l if l.starts_with("public parameters indices :") => {
                if let Some(stripped) = l.strip_prefix("public parameters indices :").map(|s| s.trim()) {
                    instructions.push(Instruction{instruction_type: InstructionType::PublicParametersIndices, instruction_body: stripped});
                }
            },
            l if l.starts_with("return value indices :") => {
                if let Some(stripped) = l.strip_prefix("return value indices :").map(|s| s.trim()) {
                    instructions.push(Instruction{instruction_type: InstructionType::ReturnValueIndices, instruction_body: stripped});
                }
            },
            
            _ => {
                continue;
            }
        }
    }
    instructions
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircuitDescription{
    pub current_witness_index: u32, 
    pub expression_width: ExpressionWidth, 
    pub private_parameters: BTreeSet<Witness>, 
    pub public_parameters: PublicInputs, 
    pub return_values: PublicInputs, 
}


impl CircuitDescription {
    fn new() -> Self{
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
     let mut public_parameters:BTreeSet<Witness> = BTreeSet::new();
     let mut return_values:BTreeSet<Witness>  = BTreeSet::new();
    // we match the lines to fill headers up these values 
    for instruction in serialized_acir{
        let parse_indices = |s: &str| -> Vec<u32> {
            let elements = s.strip_prefix("[").unwrap().strip_suffix("]").unwrap();
            if elements.is_empty(){
                return vec![];
            }
            elements.split(',')
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
            },
            InstructionType::PrivateParametersIndices => {
                // the private parameter indices is a string of form [_0, _1, _2, ...]
                // we need to split these by comma and cast each to a u32 
                let indices = parse_indices(instruction.instruction_body); 
                private_parameters.extend(indices.into_iter().map(|i| Witness::from(i)));
            },
            InstructionType::PublicParametersIndices => {
                // same as above 
                let indices = parse_indices(instruction.instruction_body);
                public_parameters.extend(indices.into_iter().map(|i| Witness::from(i)));
            },
            InstructionType::ReturnValueIndices => {
                let indices = parse_indices(instruction.instruction_body); 
                return_values.extend(indices.into_iter().map(|i| Witness::from(i)));
            },
            _ => continue
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
    Constant(F)
    
}

pub fn parse_arithmetic_expression<F: AcirField>(instruction: Instruction) -> Result<Expression<F>, String>{
    if instruction.instruction_type != InstructionType::Expr{
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
    let cleaned = expression_body.trim().strip_prefix("[").unwrap().strip_suffix("]").unwrap().trim();
    
    let re = Regex::new(r"\(([^)]+)\)").unwrap();
    // Collect all matches (terms in parentheses)
    let terms: Vec<&str> = re
        .find_iter(cleaned)
        .map(|m| m.as_str())
        .collect();
    
    // Get the constant term (if any) by splitting on whitespace and taking the last term
    let constant = cleaned.split_whitespace().last().unwrap();
    for term in terms {
        let temp: Vec<&str> = term.strip_suffix(")").unwrap().strip_prefix("(").unwrap().trim().split(",").into_iter().map(|a| a.trim()).collect();
        match temp.len(){
            3 => {
                // this is a mul term of form (constant, witness , witness)
                // we first get the witness indices 
                let first_index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap(); 
                println!("second_index: {:?}", temp[2]); 
                let second_index = temp[2].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                let coeff = parse_str_to_field(temp[0]).unwrap(); 
                mul_terms.push((coeff, Witness(first_index) , Witness(second_index)))
            }
            2 => {
                // this is a linear_combination term of form (constant, witness)
                let index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap(); 
                let coeff: F = parse_str_to_field(temp[0]).unwrap(); 
                linear_terms.push((coeff, Witness(index))); 
            }
            _ => {return Err("the expression has incorrect terms".to_string());}
        }
    }

    let q_c = parse_str_to_field(constant).unwrap(); 
    Ok(Expression{
            mul_terms: mul_terms, 
            linear_combinations: linear_terms, 
            q_c: q_c
    })
}


// pub fn build_circuit<F>(serialized_acir: Vec<(&str, &str)>) -> Circuit<F> where F: AcirField{
//     // Circuit::empty()
// }

#[cfg(test)]
mod test{
    use super::*; 
    use acir::FieldElement;

    #[test]
    fn test_serialize_acir(){
        let acir_string = "func 0
        current witness index : _1
        private parameters indices : [_0]
        public parameters indices : []
        return value indices : [_1]
        EXPR [ (-2, _0) (1, _1) 0 ]"; 

        println!("serialized input: {:?}", serialize_acir(acir_string))
    }

    #[test]
    fn test_serialize_acir_2(){
        let acir_string = "func 0
        current witness index : _3
        private parameters indices : [_0, _1, _2]
        public parameters indices : []
        return value indices : [_3]
        EXPR [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]"; 

        println!("serialized input: {:?}", serialize_acir(acir_string));
    }

    #[test]
    fn test_parse_indices()
    {
        let parse_indices = |s: &str| -> Vec<u32> {
            s.strip_prefix("[").unwrap()
            .strip_suffix("]").unwrap()
            .split(',')
            .map(|s| s.trim().strip_prefix("_").unwrap().parse::<u32>().unwrap())
            .collect()
        };
        let indices_string_2 = "[_0]"; 
        let indices_string = "[_0, _1, _2]"; 
        let indices = parse_indices(indices_string); 
        let indices_2 = parse_indices(indices_string_2); 
        println!("{:?}", indices); 
        println!("{:?}", indices_2); 

    }

    #[test]
    fn test_get_circuit_description(){
        let acir_string = "func 0
        current witness index : _1
        private parameters indices : [_0]
        public parameters indices : []
        return value indices : [_1]
        EXPR [ (-2, _0) (1, _1) 0 ]";
        let serialized_acir = serialize_acir(acir_string);
        let circuit_description = get_circuit_description(&serialized_acir); 
        println!("circuit_description{:?}", circuit_description)
    }

    #[test]
    fn test_get_circuit_description_2(){
        let acir_string = "func 0
        current witness index : _3
        private parameters indices : [_0, _1, _2]
        public parameters indices : []
        return value indices : [_3]
        EXPR [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]"; 
        let serialized_acir = serialize_acir(acir_string);
        let circuit_description = get_circuit_description(&serialized_acir); 
        println!("circuit_description{:?}", circuit_description)
    }


    #[test]
    fn test_parsing_expression() {
        let expression_body = "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]";
        
        // Remove brackets and trim
        let cleaned = expression_body.trim().strip_prefix("[").unwrap().strip_suffix("]").unwrap().trim();
        
        // Create regex to match terms in parentheses
        let re = Regex::new(r"\(([^)]+)\)").unwrap();
        
        // Collect all matches (terms in parentheses)
        let terms: Vec<&str> = re
            .find_iter(cleaned)
            .map(|m| m.as_str())
            .collect();
        
        // Get the constant term (if any) by splitting on whitespace and taking the last term
        let constant = cleaned.split_whitespace().last().unwrap();
        
        println!("Terms: {:?}", terms);        // Will print: ["(-1, _0, _2)", "(-1, _1, _2)", "(1, _3)"]
        println!("Constant: {:?}", constant);  // Will print: "0"
    }

    #[test]
    fn test_parse_arithmetic_expression(){
        let arithmetic_expression: Instruction = Instruction {instruction_type: InstructionType::Expr, instruction_body: "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]"}; 
        let expression = parse_arithmetic_expression::<FieldElement>(arithmetic_expression).unwrap(); 
        println!("expression: {:?}", expression); 
    }
}