use regex::Regex;

use super::Instruction;
use crate::circuit::Opcode;
use crate::circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};
use crate::circuit::opcodes::AcirFunctionId;
use crate::native_types::{Expression, Witness};
use crate::parser::{InstructionType, brillig_call_parser::BrilligCallParser, parse_str_to_field};
pub use acir_field::AcirField;


pub struct MemParser {}

impl MemParser {
    todo!(); 
}

