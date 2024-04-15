mod circuit;
mod config;

use super::{
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, InstructionId},
    },
    ssa_gen::Ssa,
};
use acvm::FieldElement;
pub use circuit::Plonky2Circuit;
use plonky2::{field::types::Field, iop::target::Target, plonk::circuit_data::CircuitConfig};
use std::collections::HashMap;

use self::config::{P2Builder, P2Config, P2Field};

use crate::errors::Plonky2GenError;
use crate::ssa::ir::{
    instruction::Instruction,
    types::NumericType,
    types::Type,
    value::{Value, ValueId},
};

struct P2Value {
    target: Target,
}

pub(crate) struct Builder {
    builder: P2Builder,
    translation: HashMap<ValueId, P2Value>,
}

impl Builder {
    pub(crate) fn new() -> Builder {
        let config = CircuitConfig::standard_recursion_config();
        Builder { builder: P2Builder::new(config), translation: HashMap::new() }
    }

    pub(crate) fn build(
        mut self,
        ssa: Ssa,
        parameter_names: Vec<String>,
    ) -> Result<Plonky2Circuit, Plonky2GenError> {
        let main_function =
            ssa.functions.into_values().find(|value| value.name() == "main").unwrap();
        let entry_block_id = main_function.entry_block();
        let entry_block = main_function.dfg[entry_block_id].clone();
        let parameters = entry_block
            .parameters()
            .iter()
            .map(|value_id| self.add_parameter(&main_function.dfg, *value_id))
            .collect();
        for instruction_id in entry_block.instructions() {
            match self.add_instruction(&main_function.dfg, *instruction_id) {
                Err(error) => return Err(error),
                Ok(_) => (),
            }
        }
        let data = self.builder.build::<P2Config>();
        Ok(Plonky2Circuit { data, parameters, parameter_names })
    }

    fn add_parameter(&mut self, dfg: &DataFlowGraph, value_id: ValueId) -> Target {
        let value = dfg[value_id].clone();
        let p2value = match value {
            Value::Param { block: _, position: _, typ } => match typ {
                Type::Numeric(numeric_type) => match numeric_type {
                    NumericType::Unsigned { .. } => {
                        P2Value { target: self.builder.add_virtual_target() }
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        };

        let result = p2value.target;
        self.set(value_id, p2value);
        result
    }

    /// Converts from ssa::ir::instruction::BinaryOp to the equivalent P2Builder instruction, when
    /// such conversion is straightforward.
    fn simple_convert(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut P2Builder, Target, Target) -> Target,
    ) -> P2Value {
        let a = self.get_target(lhs);
        let b = self.get_target(rhs);
        let target = p2builder_op(&mut self.builder, a, b);
        P2Value { target }
    }

    fn add_instruction(
        &mut self,
        dfg: &DataFlowGraph,
        instruction_id: InstructionId,
    ) -> Result<(), Plonky2GenError> {
        let instruction = dfg[instruction_id].clone();

        match instruction {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let p2value = match operator {
                    super::ir::instruction::BinaryOp::Mul => {
                        self.simple_convert(lhs, rhs, P2Builder::mul)
                    }

                    super::ir::instruction::BinaryOp::Div => {
                        self.simple_convert(lhs, rhs, P2Builder::div)
                    }

                    _ => {
                        let feature_name = format!("operator {}", operator);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };

                let destinations: Vec<_> =
                    dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], p2value);
            }

            Instruction::Constrain(lhs, rhs, _) => {
                let a = self.get_target(lhs);
                let b = self.get_target(rhs);
                self.builder.connect(a, b);
            }

            Instruction::RangeCheck { value, max_bit_size, assert_message: _ } => {
                let x = self.get_target(value);
                self.builder.range_check(x, usize::try_from(max_bit_size).unwrap());
            }

            _ => {
                let feature_name = format!(
                    "instruction {:?} <- {:?}",
                    dfg.instruction_results(instruction_id),
                    instruction
                );
                return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
            }
        }
        Ok(())
    }

    fn set(&mut self, value_id: ValueId, value: P2Value) {
        self.translation.insert(value_id, value);
    }

    fn get(&mut self, value_id: ValueId) -> &P2Value {
        self.translation.get(&value_id).unwrap()
    }

    fn get_target(&mut self, value_id: ValueId) -> Target {
        self.get(value_id).target
    }
}

pub(crate) fn noir_to_plonky2_field(field: FieldElement) -> P2Field {
    // TODO(plonky2): Noir doesn't support the Goldilock field. FieldElement is 254 bit, so if the
    // user enters a large integer this will fail.
    //
    // TODO(plonky2): Consider negative numbers.
    P2Field::from_canonical_u64(field.to_u128() as u64)
}
