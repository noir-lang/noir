mod circuit;
mod config;
mod div_generator;

use super::{
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, InstructionId},
    },
    ssa_gen::Ssa,
};
use acvm::FieldElement;
pub use circuit::Plonky2Circuit;
use div_generator::VariableIntDivGenerator;
use plonky2::{
    field::types::Field, iop::target::BoolTarget, iop::target::Target,
    plonk::circuit_data::CircuitConfig,
};
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
    target: P2Target,
    typ: P2Type,
}

impl P2Value {
    fn get_target(&self) -> Result<Target, Plonky2GenError> {
        Ok(match self.target {
            P2Target::IntTarget(target) => target,
            P2Target::BoolTarget(bool_target) => bool_target.target,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "get_target called on a non-int, non-bool value".to_owned(),
                })
            }
        })
    }
}

enum P2Type {
    Boolean,
    Integer(u32),
    Array,
}

enum P2Target {
    IntTarget(Target),
    BoolTarget(BoolTarget),
    ArrayTarget(u32),
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

        let mut parameters = Vec::new();
        for value_id in entry_block.parameters().iter() {
            parameters.push(self.add_parameter(&main_function.dfg, *value_id)?);
        }
        for instruction_id in entry_block.instructions() {
            match self.add_instruction(&main_function.dfg, *instruction_id) {
                Err(error) => return Err(error),
                Ok(_) => (),
            }
        }
        let data = self.builder.build::<P2Config>();
        Ok(Plonky2Circuit { data, parameters, parameter_names })
    }

    fn add_parameter(
        &mut self,
        dfg: &DataFlowGraph,
        value_id: ValueId,
    ) -> Result<Target, Plonky2GenError> {
        let value = dfg[value_id].clone();
        let p2value = match value {
            Value::Param { block: _, position: _, typ } => match typ {
                Type::Numeric(numeric_type) => match numeric_type {
                    NumericType::Unsigned { bit_size } => P2Value {
                        target: P2Target::IntTarget(self.builder.add_virtual_target()),
                        typ: P2Type::Integer(bit_size),
                    },
                    _ => {
                        return Err(Plonky2GenError::UnsupportedFeature {
                            name: "parameters that are not unsigned integers".to_owned(),
                        })
                    }
                },
                _ => {
                    return Err(Plonky2GenError::UnsupportedFeature {
                        name: "parameters that are not numeric".to_owned(),
                    })
                }
            },
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "add_parameter passed a value that is nto Value::Param".to_owned(),
                })
            }
        };

        let result = p2value.get_target();
        self.set(value_id, p2value);
        result
    }

    /// Converts from ssa::ir::instruction::BinaryOp to the equivalent P2Builder instruction, when
    /// such conversion is straightforward.
    fn convert_integer_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut P2Builder, Target, Target) -> Target,
    ) -> Result<P2Value, Plonky2GenError> {
        let (bit_size_a, target_a) = self.get_integer(lhs)?;
        let (bit_size_b, target_b) = self.get_integer(rhs)?;
        assert!(bit_size_a == bit_size_b);

        let target = p2builder_op(&mut self.builder, target_a, target_b);
        Ok(P2Value { target: P2Target::IntTarget(target), typ: P2Type::Integer(bit_size_a) })
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
                        self.convert_integer_op(lhs, rhs, P2Builder::mul)
                    }

                    super::ir::instruction::BinaryOp::Div => {
                        let (bit_size_a, target_a) = self.get_integer(lhs)?;
                        let (bit_size_b, target_b) = self.get_integer(rhs)?;
                        assert!(bit_size_a == bit_size_b);

                        let generator =
                            VariableIntDivGenerator::new(&mut self.builder, target_a, target_b);
                        self.builder.add_simple_generator(generator.clone());

                        let c = self.builder.mul(generator.quotient, target_b);
                        let d = self.builder.add(c, generator.remainder);
                        let e = self.builder.is_equal(target_a, d);
                        self.builder.assert_bool(e);

                        let f = self.builder.zero();
                        let g = self.builder.is_equal(target_b, f);
                        let h = self.builder.not(g);
                        self.builder.assert_bool(h);

                        Ok(P2Value {
                            target: P2Target::IntTarget(generator.quotient),
                            typ: P2Type::Integer(bit_size_a),
                        })
                    }

                    super::ir::instruction::BinaryOp::Add => {
                        self.convert_integer_op(lhs, rhs, P2Builder::add)
                    }

                    super::ir::instruction::BinaryOp::Sub => {
                        self.convert_integer_op(lhs, rhs, P2Builder::sub)
                    }

                    _ => {
                        let feature_name = format!("operator {}", operator);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };

                let destinations: Vec<_> =
                    dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], p2value?);
            }

            Instruction::Constrain(lhs, rhs, _) => {
                let a = self.get_target(lhs)?;
                let b = self.get_target(rhs)?;
                self.builder.connect(a, b);
            }

            Instruction::RangeCheck { value, max_bit_size, assert_message: _ } => {
                let x = self.get_target(value)?;
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

    fn get_integer(&mut self, value_id: ValueId) -> Result<(u32, Target), Plonky2GenError> {
        let value = self.get(value_id);
        let bit_size = match value.typ {
            P2Type::Integer(bit_size) => bit_size,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "lhs argument to convert_integer_op is not integer".to_owned(),
                })
            }
        };
        let target = match value.target {
            P2Target::IntTarget(target) => target,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "lhs argument to convert_integer_op has non-integer target".to_owned(),
                })
            }
        };
        Ok((bit_size, target))
    }

    /// Get the PLONKY2 target of a value, regardless of whether its type is Integer or Boolean.
    fn get_target(&mut self, value_id: ValueId) -> Result<Target, Plonky2GenError> {
        self.get(value_id).get_target()
    }
}

pub(crate) fn noir_to_plonky2_field(field: FieldElement) -> P2Field {
    // TODO(plonky2): Noir doesn't support the Goldilock field. FieldElement is 254 bit, so if the
    // user enters a large integer this will fail.
    //
    // TODO(plonky2): Consider negative numbers.
    P2Field::from_canonical_u64(field.to_u128() as u64)
}
