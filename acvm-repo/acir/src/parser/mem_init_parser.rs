use regex::Regex;

use super::Instruction;
use crate::circuit::Opcode;
use crate::circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};
use crate::circuit::opcodes::{AcirFunctionId, BlockId, BlockType};
use crate::native_types::{Expression, Witness};
use crate::parser::{InstructionType, brillig_call_parser::BrilligCallParser, parse_str_to_field};
pub use acir_field::AcirField;

pub struct MemInitParser {}

impl MemInitParser {
    pub(crate) fn parse_mem_init<F: AcirField>(
        instruction_body: &str,
    ) -> Result<Opcode<F>, String> {
        // there are 3 types of mem init instructions:
        // either of form INIT (id:{}, len:{}, witnesses:[{}])
        // INIT CALLDATA {} (id:{}, len:{}, witnesses:[{}])
        // INIT RETURNDATA (id:{}, len:{}, witnesses:[{}])
        let re = Regex::new(
            r"^INIT\s*((?:CALLDATA\s*(\d+)|RETURNDATA)?)\s*\((\d+),\s*(\d+),\s*\[(.*?)\]\)$",
        )
        .unwrap();
        let captures =
            re.captures(instruction_body).ok_or_else(|| "Failed to parse mem_init".to_string())?;

        let block_type = if captures.get(1).is_some() {
            if captures.get(1).unwrap().as_str() == "CALLDATA" {
                BlockType::CallData(captures.get(2).unwrap().as_str().parse::<u32>().unwrap())
            } else {
                BlockType::ReturnData
            }
        } else {
            BlockType::Memory
        };
        let id = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
        let len = captures.get(4).unwrap().as_str().parse::<u32>().unwrap();
        let witnesses: Vec<Witness> = captures
            .get(5)
            .unwrap()
            .as_str()
            .split(',')
            .map(|s| Witness(s.trim().to_string().parse::<u32>().unwrap()))
            .collect();

        // now we create an opcode with these
        let opcode =
            Opcode::MemoryInit { block_id: BlockId(id), init: witnesses, block_type: block_type };
        Ok(opcode)
    }
}

mod test {
    use super::*;
    use crate::acir_field::FieldElement;

    #[test]
    fn test_parse_mem_init_call_data() {
        let instruction_body = "INIT CALLDATA 5 (10, 10, [1, 2, 3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        println!("opcode: {:?}", opcode);
    }

    #[test]
    fn test_parse_mem_init_return_data() {
        let instruction_body = "INIT RETURNDATA (10, 10, [1, 2, 3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        println!("opcode: {:?}", opcode);
    }

    #[test]
    fn test_parse_mem_init() {
        let instruction_body = "INIT (10, 10, [1, 2, 3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        println!("opcode: {:?}", opcode);
    }
}
