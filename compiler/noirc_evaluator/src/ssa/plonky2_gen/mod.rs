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
use div_generator::add_div_mod;
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

#[derive(Debug)]
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

    fn make_integer(bit_size: u32, target: Target) -> P2Value {
        P2Value { target: P2Target::IntTarget(target), typ: P2Type::Integer(bit_size) }
    }

    fn make_boolean(target: BoolTarget) -> P2Value {
        P2Value { target: P2Target::BoolTarget(target), typ: P2Type::Boolean }
    }

    fn make_field(target: Target) -> P2Value {
        P2Value { target: P2Target::IntTarget(target), typ: P2Type::Field }
    }
}

#[derive(Debug)]
enum P2Type {
    Boolean,
    Integer(u32),
    Array,
    Field,
}

#[derive(Debug)]
enum P2Target {
    IntTarget(Target),
    BoolTarget(BoolTarget),
    ArrayTarget(u32),
}

pub(crate) struct Builder {
    builder: P2Builder,
    translation: HashMap<ValueId, P2Value>,
    dfg: DataFlowGraph,
}

impl Builder {
    pub(crate) fn new() -> Builder {
        let config = CircuitConfig::standard_recursion_config();
        Builder {
            builder: P2Builder::new(config),
            translation: HashMap::new(),
            dfg: DataFlowGraph::default(),
        }
    }

    pub(crate) fn build(
        mut self,
        ssa: Ssa,
        parameter_names: Vec<String>,
    ) -> Result<Plonky2Circuit, Plonky2GenError> {
        let main_function =
            ssa.functions.into_values().find(|value| value.name() == "main").unwrap();
        let entry_block_id = main_function.entry_block();
        self.dfg = main_function.dfg;
        let entry_block = self.dfg[entry_block_id].clone();

        let mut parameters = Vec::new();
        for value_id in entry_block.parameters().iter() {
            parameters.push(self.add_parameter(*value_id)?);
        }
        for instruction_id in entry_block.instructions() {
            match self.add_instruction(*instruction_id) {
                Err(error) => return Err(error),
                Ok(_) => (),
            }
        }
        let data = self.builder.build::<P2Config>();
        Ok(Plonky2Circuit { data, parameters, parameter_names })
    }

    fn add_parameter(&mut self, value_id: ValueId) -> Result<Target, Plonky2GenError> {
        let value = self.dfg[value_id].clone();
        let p2value = match value {
            Value::Param { block: _, position: _, typ } => match typ {
                Type::Numeric(numeric_type) => match numeric_type {
                    NumericType::NativeField => {
                        P2Value::make_field(self.builder.add_virtual_target())
                    }
                    NumericType::Unsigned { bit_size } => {
                        P2Value::make_integer(bit_size, self.builder.add_virtual_target())
                    }
                    _ => {
                        let feature_name = format!("parameters of type {numeric_type}");
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
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
        Ok(P2Value::make_integer(bit_size_a, target))
    }

    fn convert_bitwise_logical_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut P2Builder, BoolTarget, BoolTarget) -> BoolTarget,
    ) -> Result<P2Value, Plonky2GenError> {
        let (bit_size_a, target_a) = self.get_integer(lhs)?;
        let (bit_size_b, target_b) = self.get_integer(rhs)?;
        assert!(bit_size_a == bit_size_b);
        let bit_size = usize::try_from(bit_size_a).unwrap();

        let a_bits = self.builder.split_le(target_a, bit_size);
        let b_bits = self.builder.split_le(target_b, bit_size);

        let mut result_bits = Vec::new();
        for (i, (a_bit, b_bit)) in a_bits.iter().zip(b_bits).enumerate() {
            let result_bit = p2builder_op(&mut self.builder, *a_bit, b_bit);

            let zero = self.builder.zero();
            let one = self.builder.one();
            let two = self.builder.two();
            let result_power_of_two = if i > 0 {
                let bit = self.builder._if(result_bit, two, zero);
                self.builder.exp_u64(bit, u64::try_from(i).unwrap())
            } else {
                self.builder._if(result_bit, one, zero)
            };
            result_bits.push(result_power_of_two);
        }

        let target = self.builder.add_many(result_bits);

        Ok(P2Value::make_integer(bit_size_a, target))
    }

    fn add_instruction(&mut self, instruction_id: InstructionId) -> Result<(), Plonky2GenError> {
        let instruction = self.dfg[instruction_id].clone();

        match instruction {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let p2value = match operator {
                    super::ir::instruction::BinaryOp::Mul => {
                        self.convert_integer_op(lhs, rhs, P2Builder::mul)
                    }

                    super::ir::instruction::BinaryOp::Div => {
                        self.convert_integer_op(lhs, rhs, |builder, t1, t2| {
                            add_div_mod(builder, t1, t2).0
                        })
                    }

                    super::ir::instruction::BinaryOp::Mod => {
                        self.convert_integer_op(lhs, rhs, |builder, t1, t2| {
                            add_div_mod(builder, t1, t2).1
                        })
                    }

                    super::ir::instruction::BinaryOp::Add => {
                        self.convert_integer_op(lhs, rhs, P2Builder::add)
                    }

                    super::ir::instruction::BinaryOp::Sub => {
                        self.convert_integer_op(lhs, rhs, P2Builder::sub)
                    }

                    super::ir::instruction::BinaryOp::Eq => {
                        let target_a = self.get_target(lhs)?;
                        let target_b = self.get_target(rhs)?;
                        let target = self.builder.is_equal(target_a, target_b);
                        Ok(P2Value::make_boolean(target))
                    }

                    super::ir::instruction::BinaryOp::Lt => {
                        let (bit_size_a, target_a) = self.get_integer(lhs)?;
                        let (bit_size_b, target_b) = self.get_integer(rhs)?;
                        assert!(bit_size_a == bit_size_b);

                        let div = add_div_mod(&mut self.builder, target_a, target_b).0;
                        let zero = self.builder.zero();
                        let target = self.builder.is_equal(div, zero);
                        Ok(P2Value::make_boolean(target))
                    }

                    super::ir::instruction::BinaryOp::Xor => {
                        fn one_bit_xor(
                            builder: &mut P2Builder,
                            lhs: BoolTarget,
                            rhs: BoolTarget,
                        ) -> BoolTarget {
                            let not_lhs = builder.not(lhs);
                            let not_rhs = builder.not(rhs);
                            let c = builder.and(lhs, not_rhs);
                            let d = builder.and(not_lhs, rhs);
                            builder.or(c, d)
                        }
                        self.convert_bitwise_logical_op(lhs, rhs, one_bit_xor)
                    }

                    super::ir::instruction::BinaryOp::And => {
                        self.convert_bitwise_logical_op(lhs, rhs, P2Builder::and)
                    }

                    _ => {
                        let feature_name = format!("operator {}", operator);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
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
                    self.dfg.instruction_results(instruction_id),
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

    fn get(&mut self, value_id: ValueId) -> Option<&P2Value> {
        self.translation.get(&value_id)
    }

    fn get_integer(&mut self, value_id: ValueId) -> Result<(u32, Target), Plonky2GenError> {
        let p2value: P2Value;
        let p2value_ref = match self.get(value_id) {
            Some(p2value) => p2value,
            None => {
                let value = self.dfg[value_id].clone();
                p2value = self.create_p2value(value)?;
                &p2value
            }
        };

        let bit_size = match p2value_ref.typ {
            P2Type::Integer(bit_size) => bit_size,
            _ => {
                let message =
                    format!("lhs argument to convert_integer_op is of type {:?}", p2value_ref.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let target = match p2value_ref.target {
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
        match self.get(value_id) {
            Some(p2value) => p2value.get_target(),
            None => {
                let value = self.dfg[value_id].clone();
                self.create_p2value(value)?.get_target()
            }
        }
    }

    fn create_p2value(&mut self, value: Value) -> Result<P2Value, Plonky2GenError> {
        let (target, typ) = match value {
            Value::Param { block: _, position: _, typ } => (self.builder.add_virtual_target(), typ),
            Value::NumericConstant { constant, typ } => {
                (self.builder.constant(noir_to_plonky2_field(constant)), typ)
            }
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: format!("create_p2value passed a value that is {:?}", value),
                })
            }
        };

        match typ {
            Type::Numeric(numeric_type) => match numeric_type {
                NumericType::NativeField => Ok(P2Value::make_field(target)),
                NumericType::Unsigned { bit_size } => Ok(P2Value::make_integer(bit_size, target)),
                _ => {
                    let feature_name = format!("parameters of type {numeric_type}");
                    Err(Plonky2GenError::UnsupportedFeature { name: feature_name })
                }
            },
            _ => Err(Plonky2GenError::UnsupportedFeature {
                name: "parameters that are not numeric".to_owned(),
            }),
        }
    }
}

pub(crate) fn noir_to_plonky2_field(field: FieldElement) -> P2Field {
    // TODO(plonky2): Noir doesn't support the Goldilock field. FieldElement is 254 bit, so if the
    // user enters a large integer this will fail.
    //
    // TODO(plonky2): Consider negative numbers.
    P2Field::from_canonical_u64(field.to_u128() as u64)
}
