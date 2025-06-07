use regex::Regex;

use crate::circuit::Opcode;
use crate::circuit::opcodes::{BlockId, BlockType};
use crate::native_types::Witness;
use acir_field::AcirField;

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
            r"^INIT\s*((?:CALLDATA\s*(\d+)|RETURNDATA)?)\s*\(\s*id:\s*(\d+),\s*len:\s*(\d+),\s*witnesses:\s*\[(.*?)\]\s*\)$",
        )
        .unwrap();

        let captures =
            re.captures(instruction_body).ok_or_else(|| "Failed to parse mem_init".to_string())?;

        let block_type = if captures.get(1).is_some() {
            if captures.get(1).unwrap().as_str().starts_with("CALLDATA") {
                BlockType::CallData(captures.get(2).unwrap().as_str().parse::<u32>().unwrap())
            } else if captures.get(1).unwrap().as_str().starts_with("RETURNDATA") {
                BlockType::ReturnData
            } else {
                BlockType::Memory
            }
        } else {
            BlockType::Memory
        };
        let id = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
        let len = captures.get(4).unwrap().as_str().parse::<u32>().unwrap();
        let witnesses_string = captures.get(5).unwrap().as_str();
        let witnesses: Vec<Witness> = captures
            .get(5)
            .unwrap()
            .as_str()
            .split(',')
            .map(|s| Witness(s.trim().strip_prefix("_").unwrap().parse::<u32>().unwrap()))
            .collect();

        // now we create an opcode with these
        let opcode =
            Opcode::MemoryInit { block_id: BlockId(id), init: witnesses, block_type: block_type };
        Ok(opcode)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::acir_field::FieldElement;

    #[test]
    fn test_parse_mem_init_call_data() {
        let instruction_body = "INIT CALLDATA 5 (id: 10, len: 10, witnesses: [_1, _2, _3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        assert_eq!(
            opcode,
            Opcode::MemoryInit {
                block_id: BlockId(10),
                init: vec![Witness(1), Witness(2), Witness(3)],
                block_type: BlockType::CallData(5)
            }
        );
    }

    #[test]
    fn test_parse_mem_init_return_data() {
        let instruction_body = "INIT RETURNDATA (id: 10, len: 10, witnesses: [_1, _2, _3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        println!("opcode: {:?}", opcode);
    }

    #[test]
    fn test_parse_mem_init() {
        let instruction_body = "INIT (id: 10, len: 10, witnesses: [_1, _2, _3])";
        let opcode = MemInitParser::parse_mem_init::<FieldElement>(&instruction_body).unwrap();
        println!("opcode: {:?}", opcode);
    }
}
