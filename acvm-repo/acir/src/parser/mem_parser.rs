use regex::Regex;

use super::Instruction;
use crate::circuit::Opcode;
use crate::circuit::opcodes::{BlockId, MemOp};
use crate::native_types::{Expression, Witness};
use crate::parser::{InstructionType, brillig_call_parser::BrilligCallParser, parse_str_to_field};
use acir_field::AcirField;

pub(crate) struct MemParser {}

impl MemParser {
    pub(crate) fn parse_mem_op<F: AcirField>(
        instruction: &Instruction,
    ) -> Result<Opcode<F>, String> {
        // block_id; BlockId
        // op: MemOp<F>
        // predicate: Option<Expression<F>>
        // the serialized format looks something like:
        // MEM (id: 2, read at: EXPR [ (1, _7) 0 ], value: EXPR [ (1, _17) 0 ])
        if instruction.instruction_type != InstructionType::MemoryOp {
            return Err(format!(
                "Expected a memory operation instruction, got: {:?}",
                instruction.instruction_type
            ));
        }
        let mem_op_str = &instruction.instruction_body;
        let mem_op_regex = Regex::new(r"^MEM\s*(?:(PREDICATE\s*=\s*\[.*?\]\s*))?\s*\(id:\s*(\d+),\s*(read\s+at:|write|op\s+EXPR\s*\[.*?\])\s*(EXPR\s*\[.*?\]),\s*value:\s*(EXPR\s*\[.*?\])\)").unwrap();

        let captures =
            mem_op_regex.captures(mem_op_str).ok_or("Invalid memory operation format")?;
        let predicate_str = captures.get(1).map(|m| Some(m.as_str())).unwrap_or(None);
        let block_id = captures[2].parse::<u32>().unwrap();
        let op_type_str = captures[3].to_string();
        let read_write_at = captures[4].to_string();
        let value = captures[5].to_string();

        // we parse the location of the read/write
        let expression_regex = Regex::new(r"\(([^)]+)\)").unwrap();
        // Collect all matches (terms in parentheses)
        let terms: Vec<&str> =
            expression_regex.find_iter(read_write_at.as_str()).map(|m| m.as_str()).collect();
        let mut mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
        let mut linear_terms: Vec<(F, Witness)> = Vec::new();
        // Get the constant term (if any) by splitting on whitespace and taking the last term
        let constant = read_write_at.strip_suffix("]").unwrap().split_whitespace().last().unwrap();

        for term in terms {
            let temp: Vec<&str> = term
                .strip_suffix(")")
                .unwrap()
                .strip_prefix("(")
                .unwrap()
                .trim()
                .split(",")
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
            // now we create an expression for the read/write at
        }
        let index_expr: Expression<F> = Expression {
            mul_terms,
            linear_combinations: linear_terms,
            q_c: parse_str_to_field(constant).unwrap(),
        };

        // we do the exact same for the value
        let value_terms: Vec<&str> =
            expression_regex.find_iter(value.as_str()).map(|m| m.as_str()).collect();
        let mut value_mul_terms: Vec<(F, Witness, Witness)> = Vec::new();
        let mut value_linear_terms: Vec<(F, Witness)> = Vec::new();
        let value_constant = value.strip_suffix("]").unwrap().split_whitespace().last().unwrap();

        for term in value_terms {
            let temp: Vec<&str> = term
                .strip_suffix(")")
                .unwrap()
                .strip_prefix("(")
                .unwrap()
                .trim()
                .split(",")
                .map(|a| a.trim())
                .collect();
            match temp.len() {
                3 => {
                    // this is a mul term of form (constant, witness , witness)
                    let first_index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                    let second_index = temp[2].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                    let coeff = parse_str_to_field(temp[0]).unwrap();
                    value_mul_terms.push((coeff, Witness(first_index), Witness(second_index)));
                }
                2 => {
                    // this is a linear_combination term of form (constant, witness)
                    let index = temp[1].strip_prefix("_").unwrap().parse::<u32>().unwrap();
                    let coeff: F = parse_str_to_field(temp[0]).unwrap();
                    value_linear_terms.push((coeff, Witness(index)));
                }
                _ => {
                    return Err("the expression has incorrect terms".to_string());
                }
            }
        }
        let value_expr: Expression<F> = Expression {
            mul_terms: value_mul_terms,
            linear_combinations: value_linear_terms,
            q_c: parse_str_to_field(value_constant).unwrap(),
        };

        // we check whether the op_type_str is read at, write or op
        let mem_op = match op_type_str.as_str() {
            "read at:" => {
                // we have to make an expression equal to 0
                let read_at_expr: Expression<F> = Expression::zero();
                MemOp::<F> { operation: read_at_expr, index: index_expr, value: value_expr }
            }
            "write" => {
                // we have to make an expression equal to 1
                let write_expr: Expression<F> = Expression::one();
                MemOp::<F> { operation: write_expr, index: index_expr, value: value_expr }
            }

            _ => return Err(format!("Invalid memory operation format: {}", op_type_str)),
        };

        let predicate = predicate_str
            .map(|predicate_str| BrilligCallParser::parse_predicate::<F>(predicate_str).unwrap());

        Ok(Opcode::MemoryOp { block_id: BlockId(block_id), op: mem_op, predicate })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::acir_field::FieldElement;

    #[test]
    fn test_mem_op_write_parser() {
        let mem_op_str = "MEM (id: 2, write EXPR [ (1, _7) 0 ], value: EXPR [ (1, _17) 0 ])";
        let mem_op_instruction = Instruction {
            instruction_type: InstructionType::MemoryOp,
            instruction_body: mem_op_str,
        };
        let mem_op_opcode = MemParser::parse_mem_op::<FieldElement>(&mem_op_instruction).unwrap();
        let expected_opcode = Opcode::MemoryOp {
            block_id: BlockId(2),
            op: MemOp::write_to_mem_index(
                Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![(FieldElement::one(), Witness(7))],
                    q_c: FieldElement::zero(),
                },
                Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![(FieldElement::one(), Witness(17))],
                    q_c: FieldElement::zero(),
                },
            ),
            predicate: None,
        };
        assert_eq!(mem_op_opcode, expected_opcode);
    }

    #[test]
    fn test_mem_op_read_parser() {
        let mem_op_str = "MEM (id: 2, read at: EXPR [ (1, _7) 0 ], value: EXPR [ (1, _17) 0 ])";
        let mem_op_instruction = Instruction {
            instruction_type: InstructionType::MemoryOp,
            instruction_body: mem_op_str,
        };
        let mem_op_opcode = MemParser::parse_mem_op::<FieldElement>(&mem_op_instruction).unwrap();
        let expected_opcode = Opcode::MemoryOp {
            block_id: BlockId(2),
            op: MemOp::read_at_mem_index(
                Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![(FieldElement::one(), Witness(7))],
                    q_c: FieldElement::zero(),
                },
                Witness(17),
            ),
            predicate: None,
        };
        assert_eq!(mem_op_opcode, expected_opcode);
    }
}
