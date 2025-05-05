use crate::instruction::{Argument, Instruction};
use noir_ssa_fuzzer::typed_value::ValueType;
use std::collections::HashSet;
use std::mem::{Discriminant, discriminant};
#[derive(Default)]
pub struct FuzzerOptions {
    pub max_instructions: u32,
    pub max_instructions_per_block: u32,
    /// List of instructions that are forbidden to be used in the fuzzer
    pub totally_forbidden_instructions: HashSet<Discriminant<Instruction>>,
    pub constant_execution_enabled: bool,
}

impl FuzzerOptions {
    pub fn new(
        max_instructions: u32,
        max_instructions_per_block: u32,
        totally_forbidden_instructions: HashSet<Discriminant<Instruction>>,
        constant_execution_enabled: bool,
    ) -> Self {
        Self {
            max_instructions,
            max_instructions_per_block,
            totally_forbidden_instructions,
            constant_execution_enabled,
        }
    }

    pub fn is_forbidden(&self, instruction: Instruction) -> bool {
        self.totally_forbidden_instructions.iter().any(|i| *i == discriminant(&instruction))
    }

    fn get_default_totally_forbidden_instructions() -> HashSet<Discriminant<Instruction>> {
        return HashSet::from([
            discriminant(&Instruction::Shl {
                lhs: Argument { index: 0, value_type: ValueType::Boolean },
                rhs: Argument { index: 0, value_type: ValueType::Boolean },
            }),
            discriminant(&Instruction::Shr {
                lhs: Argument { index: 0, value_type: ValueType::Boolean },
                rhs: Argument { index: 0, value_type: ValueType::Boolean },
            }),
            discriminant(&Instruction::Xor {
                lhs: Argument { index: 0, value_type: ValueType::Boolean },
                rhs: Argument { index: 0, value_type: ValueType::Boolean },
            }),
            discriminant(&Instruction::Or {
                lhs: Argument { index: 0, value_type: ValueType::Boolean },
                rhs: Argument { index: 0, value_type: ValueType::Boolean },
            }),
            discriminant(&Instruction::And {
                lhs: Argument { index: 0, value_type: ValueType::Boolean },
                rhs: Argument { index: 0, value_type: ValueType::Boolean },
            }),
        ]);
    }

    pub fn default() -> Self {
        Self {
            max_instructions: 100,
            max_instructions_per_block: 10,
            totally_forbidden_instructions: Self::get_default_totally_forbidden_instructions(),
            constant_execution_enabled: false,
        }
    }
}
