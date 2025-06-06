use crate::circuit::Opcode;
use crate::native_types::{Expression, Witness};
use crate::parser::parse_str_to_field;
use crate::parser::{Instruction, InstructionType};
use acir_field::AcirField;
use regex::Regex;

pub(crate) struct ArithmeticParser {}

impl ArithmeticParser {
    pub(crate) fn parse_arithmetic_instruction<F: AcirField>(
        instruction: Instruction,
    ) -> Result<Opcode<F>, String> {
        if instruction.instruction_type != InstructionType::Expr {
            return Err(format!(
                "Expected Expr instruction, got {:?}",
                instruction.instruction_type
            ));
        }
        let expression_body = instruction.instruction_body;
        // the expression body is of form [ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]
        // which corresponds to expression 0 - w0 * w2 - w1 * w2 + w_3 = 0
        // the elements with 3 elements are mul_terms and the elements with 1 element are linear terms
        let mut mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
        let mut linear_terms: Vec<(F, Witness)> = Vec::new();
        let q_c: F = F::zero();
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
        Ok(Opcode::AssertZero(Expression {
            mul_terms: mul_terms,
            linear_combinations: linear_terms,
            q_c: q_c,
        }))
    }
}

#[cfg(test)]
mod test {
    use acir_field::AcirField;
    use regex::Regex;

    use crate::{
        acir_field::FieldElement,
        circuit::Opcode,
        native_types::{Expression, Witness},
        parser::{Instruction, InstructionType},
    };

    use super::ArithmeticParser;

    #[test]
    fn test_parse_arithmetic_expression() {
        let arithmetic_expression: Instruction = Instruction {
            instruction_type: InstructionType::Expr,
            instruction_body: "[ (-1, _0, _2) (-1, _1, _2) (1, _3) 0 ]",
        };
        let expression =
            ArithmeticParser::parse_arithmetic_instruction::<FieldElement>(arithmetic_expression)
                .unwrap();
        assert_eq!(
            expression,
            Opcode::AssertZero(Expression {
                mul_terms: vec![
                    (-FieldElement::one(), Witness(0), Witness(2)),
                    (-FieldElement::one(), Witness(1), Witness(2))
                ],
                linear_combinations: vec![(FieldElement::one(), Witness(3))],
                q_c: FieldElement::zero()
            })
        );
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
}
