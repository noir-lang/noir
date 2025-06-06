use regex::Regex;

use super::Instruction;
use crate::circuit::Opcode;
use crate::circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};
use crate::native_types::{Expression, Witness};
use crate::parser::{InstructionType, parse_str_to_field};
use acir_field::AcirField;

pub struct BrilligCallParser {}

impl BrilligCallParser {
    fn deserialize_brillig_call<'a>(
        instruction: &'a Instruction<'a>,
    ) -> Result<(&'a str, &'a str, u32, Option<&'a str>), String> {
        if instruction.instruction_type != InstructionType::BrilligCall {
            return Err(format!(
                "Expected BRILLIG_CALL instruction, got {:?}",
                instruction.instruction_type
            ));
        }

        let instruction_body = instruction.instruction_body;
        let re = Regex::new(r"func (\d+):\s*(?:(PREDICATE\s*=\s*EXPR\s*\[[^]]*\]\s*))?\s*inputs:\s*\[(.*?)\],\s*outputs:\s*\[(.*?)\];").unwrap();
        let captures = re
            .captures(instruction_body)
            .ok_or_else(|| "Failed to parse Brillig call".to_string())?;
        let id = captures.get(1).unwrap().as_str();
        // convert id to u32
        let id = id.parse::<u32>().unwrap();
        let predicate = captures.get(2).is_some().then(|| captures.get(2).unwrap().as_str());
        let inputs = captures.get(3).unwrap().as_str();
        let outputs = captures.get(4).unwrap().as_str();
        Ok((inputs, outputs, id, predicate))
    }

    pub(crate) fn parse_brillig_inputs<F: AcirField>(
        call_inputs_string: &str,
    ) -> Result<Vec<BrilligInputs<F>>, String> {
        // the inputs are of 3 types: Single(Expression), Array(Vec<Expression>) and MemoryArray(BlockId)
        // we keep 3 different vectors to store each type

        // we don't want to be using two different array inputs because it's messing up the order of inputs
        // so we want one regex for both array and single inputs and have a match statement to handle the different cases
        let mut inputs_expressions: Vec<BrilligInputs<F>> = Vec::new();
        let input_regex = Regex::new(r"Single\(Expression\s*\{[^}]*\}\)|Array\(\[Expression\s*\{[^}]*\}(?:\s*,\s*Expression\s*\{[^}]*\})*\s*\]\)").unwrap();
        let inputs =
            input_regex.find_iter(call_inputs_string).map(|m| m.as_str()).collect::<Vec<&str>>();

        // now we have to split each of the inputs. let us start with the single inputs
        // we have to create an expression for each single input
        for input in inputs.clone() {
            if input.starts_with("Single") {
                let trimmed_input =
                    input.trim().strip_prefix("Single(").unwrap().strip_suffix(")").unwrap();
                let expression =
                    Self::parse_expression::<F>(trimmed_input).map_err(|e| e.to_string())?;
                inputs_expressions.push(BrilligInputs::Single(expression));
            } else if input.starts_with("Array") {
                let trimmed_input =
                    input.trim().strip_prefix("Array(").unwrap().strip_suffix(")").unwrap();
                // now we split the array of expressions into individual expressions
                let expressions_array_regex =
                    Regex::new(r"Expression\s*\{[^}]*\}\s*(?:,\s*)?").unwrap();
                // we iterate over each expression and parse it
                let expressions = expressions_array_regex
                    .find_iter(trimmed_input)
                    .map(|m| m.as_str())
                    .collect::<Vec<&str>>();
                let mut expressions_array: Vec<Expression<F>> = Vec::new();
                for expression in expressions {
                    let expression =
                        Self::parse_expression::<F>(expression).map_err(|e| e.to_string())?;
                    expressions_array.push(expression);
                }
                inputs_expressions.push(BrilligInputs::Array(expressions_array));
            } else {
                return Err("Invalid input type".to_string());
            }
        }

        Ok(inputs_expressions)
    }

    fn parse_brillig_outputs(call_string: &str) -> Result<Vec<BrilligOutputs>, String> {
        // brillig outputs are of form Simple(Witness) or Array(Vec<Witness>)
        let mut outputs_array: Vec<BrilligOutputs> = Vec::new();
        let output_regex = Regex::new(
            r"(Simple|Array)\((Witness\(\d+\)|\[Witness\(\d+\)(?:\s*,\s*Witness\(\d+\))*\])\)",
        )
        .unwrap();
        let captures = output_regex.captures_iter(call_string).collect::<Vec<_>>();
        for capture in captures {
            let output_type = capture.get(1).unwrap().as_str();
            let content = capture.get(2).unwrap().as_str();
            match output_type {
                "Simple" => {
                    outputs_array.push(BrilligOutputs::Simple(Witness(
                        content
                            .strip_prefix("Witness(")
                            .unwrap()
                            .strip_suffix(")")
                            .unwrap()
                            .parse::<u32>()
                            .unwrap(),
                    )));
                }
                "Array" => {
                    let mut witness_array: Vec<Witness> = Vec::new();
                    let witnesses_regex = Regex::new(r"Witness\((\d+)\)").unwrap();
                    let captures = witnesses_regex.captures_iter(content).collect::<Vec<_>>();
                    for capture in captures {
                        let witness = capture.get(1).unwrap().as_str();
                        witness_array.push(Witness(witness.parse::<u32>().unwrap()));
                    }
                    outputs_array.push(BrilligOutputs::Array(witness_array));
                }
                _ => {
                    return Err("Invalid output type".to_string());
                }
            }
        }
        Ok(outputs_array)
    }

    pub(crate) fn parse_expression<F: AcirField>(
        expression_str: &str,
    ) -> Result<Expression<F>, String> {
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
        let q_c = parse_str_to_field::<F>(q_c_str).map_err(|e| e.to_string())?;
        let expression =
            Expression { mul_terms: mul_terms, linear_combinations: linear_combinations, q_c: q_c };
        Ok(expression)
    }

    pub(crate) fn parse_predicate<F: AcirField>(
        predicate_str: &str,
    ) -> Result<Expression<F>, String> {
        let cleaned = predicate_str
            .trim()
            .strip_prefix("PREDICATE = EXPR [")
            .unwrap()
            .strip_suffix("]")
            .unwrap();
        let re = Regex::new(r"\(([^)]+)\)").unwrap();
        // Collect all matches (terms in parentheses)
        let terms: Vec<&str> = re.find_iter(cleaned).map(|m| m.as_str()).collect();
        // Get the constant term (if any) by splitting on whitespace and taking the last term
        let constant = parse_str_to_field(cleaned.split_whitespace().last().unwrap()).unwrap();
        let mut mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
        let mut linear_terms: Vec<(F, Witness)> = Vec::new();
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
        let expression =
            Expression { mul_terms: mul_terms, linear_combinations: linear_terms, q_c: constant };
        Ok(expression)
    }

    pub(crate) fn parse_brillig_call<F: AcirField>(
        brillig_call_instruction: &Instruction,
    ) -> Result<Opcode<F>, String> {
        // we first serialize the call string
        let (brillig_input_string, brillig_output_string, brillig_id, predicate_string) =
            Self::deserialize_brillig_call(brillig_call_instruction).map_err(|e| e.to_string())?;
        // now we parse the inputs
        let brillig_inputs =
            Self::parse_brillig_inputs::<F>(brillig_input_string).map_err(|e| e.to_string())?;
        // now we parse the outputs
        let outputs = Self::parse_brillig_outputs(brillig_output_string);
        // now we parse the predicate
        let mut predicate = None;

        if let Some(predicate_string) = predicate_string {
            predicate =
                Some(Self::parse_predicate::<F>(predicate_string).map_err(|e| e.to_string())?);
        }
        // now we create the BrilligCall
        Ok(Opcode::BrilligCall {
            id: BrilligFunctionId(brillig_id),
            inputs: brillig_inputs,
            outputs: outputs.map_err(|e| e.to_string())?,
            predicate: predicate,
        })
    }
}

#[cfg(test)]
mod test {
    use super::BrilligCallParser;
    use crate::{
        native_types::Witness,
        parser::{
            Instruction, InstructionType,
            utils::{clean_string, parse_str_to_field},
        },
    };

    use acir_field::{AcirField, FieldElement};
    use regex::Regex;

    #[test]
    fn test_brillig_inputs_parser() {
        let inputs = "Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [(1 , Witness(0) , Witness(1)) , (1 , Witness(2) , Witness(3))], linear_combinations: [], q_c: 4294967296 }), Array([Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(1))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }])";

        let brillig_inputs =
            BrilligCallParser::parse_brillig_inputs::<FieldElement>(inputs).unwrap();
        let inputs_string = format!("{:?}", brillig_inputs);
        assert_eq!(clean_string(inputs), clean_string(inputs_string.as_str()));
    }

    #[test]
    fn test_brillig_call_parser_2() {
        let instruction = Instruction {
            instruction_type: InstructionType::BrilligCall,
            instruction_body: "func 0: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })], outputs: [Simple(Witness(4)), Simple(Witness(5))];",
        };
        let brillig_call =
            BrilligCallParser::parse_brillig_call::<FieldElement>(&instruction).unwrap();
        let brillig_call_string = format!("{:?}", brillig_call);
        let brillig_call_string_cleaned = clean_string(brillig_call_string.as_str());
        let brillig_call_string_no_prefix =
            brillig_call_string_cleaned.strip_prefix("BRILLIGCALL").unwrap();
        let brillig_call_string_expected = clean_string(instruction.instruction_body);
        assert_eq!(
            brillig_call_string_no_prefix,
            brillig_call_string_expected.strip_suffix(";").unwrap()
        );
    }

    #[test]
    fn test_mul_terms_parser() {
        let input = "(1 , Witness(0) , Witness(1)) , (-1 , Witness(2) , Witness(3))";
        let mul_terms_regex =
            Regex::new(r"\(\s*(-?\d+)\s*,\s*Witness\((\d+)\)\s*,\s*Witness\((\d+)\)\s*\)").unwrap();
        let captures = mul_terms_regex.captures_iter(input).collect::<Vec<_>>();
        let mut mull_terms: Vec<(FieldElement, Witness, Witness)> = Vec::new();
        for capture in captures {
            let q_m = capture.get(1).unwrap().as_str();
            let w_l = capture.get(2).unwrap().as_str();
            let w_r = capture.get(3).unwrap().as_str();
            let mul_term: (FieldElement, Witness, Witness) = (
                parse_str_to_field::<FieldElement>(q_m).unwrap(),
                Witness(w_l.parse::<u32>().unwrap()),
                Witness(w_r.parse::<u32>().unwrap()),
            );
            mull_terms.push(mul_term);
        }
        assert_eq!(
            mull_terms,
            vec![
                (FieldElement::one(), Witness(0), Witness(1)),
                (-FieldElement::one(), Witness(2), Witness(3))
            ]
        );
    }

    #[test]
    fn test_brillig_outputs_parser() {
        let outputs_string = "Simple(Witness(11)), Simple(Witness(12)), Array([Witness(12), Witness(13), Witness(14)])";
        let outputs = BrilligCallParser::parse_brillig_outputs(outputs_string);
        let outputs_string = format!("{:?}", outputs);
        assert_eq!(clean_string(outputs_string.as_str()), clean_string(outputs_string.as_str()));
    }

    #[test]
    fn test_brillig_call_parser() {
        let brillig_call_string = "BRILLIG CALL func 0: PREDICATE = EXPR [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ] inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Array([Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(1))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }]), Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })], outputs: [Simple(Witness(11)), Array([Witness(12), Witness(13), Witness(14)])];";

        let brillig_call_instruction = Instruction {
            instruction_type: InstructionType::BrilligCall,
            instruction_body: brillig_call_string,
        };
        let brillig_call =
            BrilligCallParser::parse_brillig_call::<FieldElement>(&brillig_call_instruction)
                .unwrap();
        let brillig_call_serialized = format!("{:}", brillig_call);
        let brillig_call_string_cleaned = clean_string(brillig_call_serialized.as_str());

        let brillig_call_string_expected = clean_string(brillig_call_instruction.instruction_body);
        assert_eq!(
            brillig_call_string_cleaned,
            brillig_call_string_expected.strip_suffix(";").unwrap()
        );
    }

    #[test]
    fn test_brillig_inputs_parser_2() {
        let input_string = "Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Array([Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(1))], q_c: 0 }, Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 }]), Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0)), (1, Witness(1))], q_c: 0 }), Single(Expression { mul_terms: [], linear_combinations: [], q_c: 4294967296 })";
        let brillig_inputs =
            BrilligCallParser::parse_brillig_inputs::<FieldElement>(input_string).unwrap();
        let inputs_string = format!("{:?}", brillig_inputs);
        assert_eq!(clean_string(input_string), clean_string(inputs_string.as_str()));
    }
}
