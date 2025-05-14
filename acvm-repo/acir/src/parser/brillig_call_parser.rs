use num_bigint::BigUint;
use num_traits::Num;
use regex::Regex;
use std::collections::BTreeSet;

pub use super::Instruction;
use crate::circuit::brillig::{BrilligInputs, BrilligOutputs};
use crate::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
use crate::circuit::{AssertionPayload, Opcode, OpcodeLocation, PublicInputs, opcodes};
use crate::native_types::{Expression, Witness};
use crate::parser::{InstructionType, parse_str_to_field};
use crate::proto::acir::circuit::opcode::{BrilligCall, Call, MemoryInit, MemoryOp};
use crate::proto::acir::circuit::{Circuit, ExpressionWidth};
pub use acir_field;
pub use acir_field::AcirField;
pub use brillig;

pub use crate::circuit::black_box_functions::BlackBoxFunc;
pub use crate::circuit::opcodes::InvalidInputBitSize;

pub struct BrilligCallParser {}

impl BrilligCallParser {
    pub(crate) fn serialize_brillig_call<'a>(
        instruction: &'a Instruction<'a>,
    ) -> Result<(&'a str, &'a str, u32), String> {
        if instruction.instruction_type != InstructionType::BrilligCall {
            return Err(format!(
                "Expected BRILLIG_CALL instruction, got {:?}",
                instruction.instruction_type
            ));
        }

        let instruction_body = instruction.instruction_body;
        let re = Regex::new(r"func (\d+):\s*inputs:\s*\[(.*?)\]\s*outputs:\s*\[(.*?)\]").unwrap();

        let captures = re
            .captures(instruction_body)
            .ok_or_else(|| "Failed to parse Brillig call".to_string())?;
        let id = captures.get(1).unwrap().as_str();
        // convert id to u32
        let id = id.parse::<u32>().unwrap();
        let inputs = captures.get(2).unwrap().as_str();
        let outputs = captures.get(3).unwrap().as_str();
        Ok((inputs, outputs, id))
    }

    pub(crate) fn parse_brillig_inputs<F: AcirField>(
        call_inputs_string: &str,
    ) -> Result<(Vec<Expression<F>>, Vec<Expression<F>>, Vec<&str>), String> {
        // the inputs are of 3 types: Single(Expression), Array(Vec<Expression>) and MemoryArray(BlockId)
        // we keep 3 different vectors to store each type
        let mut single_inputs_expressions: Vec<Expression<F>> = Vec::new();
        let single_input_regex = Regex::new(r"Single\(Expression\s*\{[^}]*\}\)").unwrap();
        let single_inputs = single_input_regex
            .find_iter(call_inputs_string)
            .map(|m| m.as_str())
            .collect::<Vec<&str>>();

        let array_input_regex =
            Regex::new(r"Array\(\[Expression\s*\{[^}]*\}(?:\s*,\s*Expression\s*\{[^}]*\})*\]\)")
                .unwrap();
        let array_inputs_str = array_input_regex
            .find_iter(call_inputs_string)
            .map(|m| m.as_str())
            .collect::<Vec<&str>>();

        let memory_array_input_regex = Regex::new(r"MemoryArray\(([^)]+)\)").unwrap();
        let memory_array_inputs = memory_array_input_regex
            .find_iter(call_inputs_string)
            .map(|m| m.as_str())
            .collect::<Vec<&str>>();

        // now we have to split each of the inputs. let us start with the single inputs
        // we have to create an expression for each single input
        for single_input in single_inputs.clone() {
            let trimmed_input =
                single_input.trim().strip_prefix("Single(").unwrap().strip_suffix(")").unwrap();
            let expression = Self::parse_expression::<F>(trimmed_input);
            single_inputs_expressions.push(expression);
        }

        // now we parse the array inputs
        // array inputs are an array of expressions, so we can use the same logic as before to parse them
        let mut array_inputs: Vec<Expression<F>> = Vec::new();
        for array_input in array_inputs_str.clone() {
            // we remove the Array( and )
            let trimmed_input =
                array_input.trim().strip_prefix("Array(").unwrap().strip_suffix(")").unwrap();
            // now we split the array of expressions into individual expressions
            let expressions_array_regex = Regex::new(r"Expression\s*\{[^}]*\}\s*,\s*").unwrap();
            // we iterate over each expression and parse it
            let expressions = expressions_array_regex
                .find_iter(trimmed_input)
                .map(|m| m.as_str())
                .collect::<Vec<&str>>();
            for expression in expressions {
                let expression = Self::parse_expression::<F>(expression);
                array_inputs.push(expression);
            }
        }

        Ok((single_inputs_expressions, array_inputs, memory_array_inputs))
    }

    fn parse_brillig_outputs(call_string: &str) -> BrilligOutputs {
        // brillig outputs are of form Simple(Witness) or Array(Vec<Witness>)
        let mut simple_outputs_array: Vec<Witness> = Vec::new();
        let mut array_outputs_array: Vec<Vec<Witness>> = Vec::new();
        let simple_outputs_regex = Regex::new(r"Simple\((Witness\((\d+)\))").unwrap();
        let captures = simple_outputs_regex.captures_iter(call_string).collect::<Vec<_>>();
        for capture in captures {
            let w = capture.get(1).unwrap().as_str();
            simple_outputs_array.push(Witness(w.parse::<u32>().unwrap()));
        }

        todo!()
    }

    fn parse_expression<F: AcirField>(expression_str: &str) -> Expression<F> {
        let single_input_regex = Regex::new(
            r"mul_terms:\s*\[(.*?)\],\s*linear_combinations:\s*\[(.*?)\],\s*q_c:\s*(\d+)",
        )
        .unwrap();
        let captures = single_input_regex.captures(expression_str).unwrap();
        let mul_terms_str = captures.get(1).unwrap().as_str();
        let linear_combinations_str = captures.get(2).unwrap().as_str();
        let q_c_str = captures.get(3).unwrap().as_str();

        // now we parse the mul_terms
        // [(F, Witness, Witness), (F, Witness, Witness), ...]
        // we iterate over each (F, Witness, Witness) and create an Expression<F>
        let mul_terms_regex =
            Regex::new(r"\(\s*(-?\d+)\s*,\s*Witness\((\d+)\)\s*,\s*Witness\((\d+)\)\s*\)").unwrap();
        let mut mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
        let captures = mul_terms_regex.captures_iter(mul_terms_str).collect::<Vec<_>>();
        for capture in captures {
            let q_m = capture.get(1).unwrap().as_str();
            let w_l = capture.get(2).unwrap().as_str();
            let w_r = capture.get(3).unwrap().as_str();
            let new_mul_term = (
                parse_str_to_field::<F>(q_m).unwrap(),
                Witness(w_l.parse::<u32>().unwrap()),
                Witness(w_r.parse::<u32>().unwrap()),
            );
            mul_terms.push(new_mul_term);
        }
        // now we parse the linear_combinations term
        // [(F, Witness), (F, Witness), ...]
        let linear_combinations_regex =
            Regex::new(r"\(\s*(-?\d+)\s*,\s*Witness\((\d+)\)\s*\)").unwrap();
        let mut linear_combinations: Vec<(F, Witness)> = Vec::new();
        let captures =
            linear_combinations_regex.captures_iter(linear_combinations_str).collect::<Vec<_>>();
        for capture in captures {
            let q_l = capture.get(1).unwrap().as_str();
            let w = capture.get(2).unwrap().as_str();
            let new_linear_combination =
                (parse_str_to_field::<F>(q_l).unwrap(), Witness(w.parse::<u32>().unwrap()));
            linear_combinations.push(new_linear_combination);
        }
        let q_c = parse_str_to_field::<F>(q_c_str).unwrap();
        let expression =
            Expression { mul_terms: mul_terms, linear_combinations: linear_combinations, q_c: q_c };
        expression
    }
}

#[test]
fn test_mul_terms_parser() {
    let input = "(1 , Witness(0) , Witness(1)) , (-1 , Witness(2) , Witness(3))";
    let mul_terms_regex =
        Regex::new(r"\(\s*(-?\d+)\s*,\s*Witness\((\d+)\)\s*,\s*Witness\((\d+)\)\s*\)").unwrap();
    let captures = mul_terms_regex.captures_iter(input).collect::<Vec<_>>();
    for capture in captures {
        let q_m = capture.get(1).unwrap().as_str();
        let w_l = capture.get(2).unwrap().as_str();
        let w_r = capture.get(3).unwrap().as_str();
        println!("q_m: {:?}", q_m);
        println!("w_l: {:?}", w_l);
        println!("w_r: {:?}", w_r);
    }
}

#[test]
fn test_brillig_outputs_parser() {
    let outputs_string = "Simple(Witness(11)), Array([Witness(12), Witness(13), Witness(14)])";
    let outputs = BrilligCallParser::parse_brillig_outputs(outputs_string);
    println!("outputs: {:?}", outputs);
}
