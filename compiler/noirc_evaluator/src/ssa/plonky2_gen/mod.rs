mod circuit;
mod config;
mod div_generator;
mod intrinsics;

use super::{
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, InstructionId, Intrinsic},
    },
    ssa_gen::Ssa,
};
use acvm::{acir::BlackBoxFunc, FieldElement};
pub use circuit::Plonky2Circuit;
use div_generator::add_div_mod;
use plonky2::{
    field::types::Field, iop::target::BoolTarget, iop::target::Target,
    plonk::circuit_data::CircuitConfig,
};
use std::collections::HashMap;

use self::config::{P2Builder, P2Config, P2Field};
use self::intrinsics::make_sha256_circuit;

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

    fn make_array(bit_size: u32, targets: Vec<Target>) -> P2Value {
        P2Value { target: P2Target::ArrayTarget(targets), typ: P2Type::Array(bit_size) }
    }

    /// Extends the given list with the Noir targets wrapped by this value: if this value is an
    /// array, there would be multiple targets wrapped.
    fn extend_parameter_list(&self, parameters: &mut Vec<Target>) -> Result<(), Plonky2GenError> {
        Ok(match self.target {
            P2Target::ArrayTarget(ref targets) => parameters.extend(targets.iter()),
            _ => parameters.push(self.get_target()?),
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum P2Type {
    Boolean,
    Integer(u32),
    Array(u32),
    Field,
}

#[derive(Debug)]
enum P2Target {
    IntTarget(Target),
    BoolTarget(BoolTarget),
    ArrayTarget(Vec<Target>),
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
            self.add_parameter(*value_id)?;
            let p2value = self.get(*value_id).unwrap();
            let _ = p2value.extend_parameter_list(&mut parameters)?;
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

    fn add_parameter(&mut self, value_id: ValueId) -> Result<(), Plonky2GenError> {
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
                Type::Array(composite_type, array_size) => {
                    if composite_type.len() != 1 {
                        let feature_name = format!("composite array type {:?}", composite_type);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                    let bit_size = composite_type[0].bit_size();
                    let targets = self.builder.add_virtual_targets(array_size);
                    P2Value::make_array(bit_size, targets)
                }
                _ => {
                    let feature_name = format!("parameters of type {typ}");
                    return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                }
            },
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "add_parameter passed a value that is nto Value::Param".to_owned(),
                })
            }
        };

        self.set(value_id, p2value);
        Ok(())
    }

    /// Converts from ssa::ir::instruction::BinaryOp to the equivalent P2Builder instruction, when
    /// such conversion is straightforward and the arguments are integers.
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

    /// Converts from ssa::ir::instruction::BinaryOp to the equivalent P2Builder instruction, when
    /// such conversion is straightforward and the arguments are booleans.
    fn convert_boolean_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut P2Builder, BoolTarget, BoolTarget) -> BoolTarget,
    ) -> Result<P2Value, Plonky2GenError> {
        let target_a = self.get_boolean(lhs)?;
        let target_b = self.get_boolean(rhs)?;

        let target = p2builder_op(&mut self.builder, target_a, target_b);
        Ok(P2Value::make_boolean(target))
    }

    fn convert_bitwise_logical_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        single_bit_op: fn(&mut P2Builder, BoolTarget, BoolTarget) -> BoolTarget,
    ) -> Result<P2Value, Plonky2GenError> {
        let (bit_size_a, target_a) = self.get_integer(lhs)?;
        let (bit_size_b, target_b) = self.get_integer(rhs)?;
        assert!(bit_size_a == bit_size_b);
        let bit_size = usize::try_from(bit_size_a).unwrap();

        let a_bits = self.builder.split_le(target_a, bit_size);
        let b_bits = self.builder.split_le(target_b, bit_size);

        let mut result_bits = Vec::new();
        for (i, (a_bit, b_bit)) in a_bits.iter().zip(b_bits).enumerate() {
            let result_bit = single_bit_op(&mut self.builder, *a_bit, b_bit);

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

    fn perform_sha256(&mut self, argument: (u32, Vec<Target>), destination: ValueId) {
        assert!(
            argument.0 == 8,
            "element size of argument to sha256 is not of size 8 but {}",
            argument.0
        );
        let msg_len = u64::try_from(argument.1.len()).unwrap() * u64::try_from(argument.0).unwrap();
        let sha256targets = make_sha256_circuit(&mut self.builder, msg_len);
        let mut j = 0;
        for target in argument.1 {
            let split_arg = self.builder.split_le(target, 8);
            for arg_bit in split_arg.iter().rev() {
                self.builder.connect(arg_bit.target, sha256targets.message[j].target);
                j += 1;
            }
        }
        j = 0;
        let mut result = Vec::new();
        while j < 256 {
            result.push(self.builder.le_sum(sha256targets.digest[j..j + 8].iter().rev()));
            j += 8;
        }
        let p2value = P2Value::make_array(8, result);
        self.set(destination, p2value);
    }

    fn add_instruction(&mut self, instruction_id: InstructionId) -> Result<(), Plonky2GenError> {
        let instruction = self.dfg[instruction_id].clone();

        match instruction {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let p2value = match operator {
                    super::ir::instruction::BinaryOp::Mul => {
                        let typ = self.get_type(lhs)?;
                        match typ {
                            P2Type::Boolean => self.convert_boolean_op(lhs, rhs, P2Builder::and),
                            P2Type::Integer(_) => self.convert_integer_op(lhs, rhs, P2Builder::mul),
                            _ => {
                                let feature_name = format!("Mul instruction on {:?}", typ);
                                return Err(Plonky2GenError::UnsupportedFeature {
                                    name: feature_name,
                                });
                            }
                        }
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

                    super::ir::instruction::BinaryOp::Or => {
                        self.convert_bitwise_logical_op(lhs, rhs, P2Builder::or)
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

            Instruction::ArrayGet { array, index } => {
                let index = self.dfg[index].clone();
                let num_index = match index {
                    Value::NumericConstant { constant, .. } => constant.to_u128() as usize,
                    _ => {
                        let feature_name = format!("indexing array with an {:?}", index);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };
                let (bit_size, target) = self.get_array_element(array, num_index)?;

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], P2Value::make_integer(bit_size, target));
            }

            Instruction::Call { func, arguments } => {
                let func = self.dfg[func].clone();

                assert!(arguments.len() == 1);
                let argument = self.get_array(arguments[0])?;

                match func {
                    Value::Intrinsic(intrinsic) => match intrinsic {
                        Intrinsic::BlackBox(bb_func) => match bb_func {
                            BlackBoxFunc::SHA256 => {
                                let destinations: Vec<_> = self
                                    .dfg
                                    .instruction_results(instruction_id)
                                    .iter()
                                    .cloned()
                                    .collect();
                                assert!(destinations.len() == 1);
                                self.perform_sha256(argument, destinations[0]);
                            }
                            _ => {
                                let feature_name = format!("black box function {:?}", bb_func);
                                return Err(Plonky2GenError::UnsupportedFeature {
                                    name: feature_name,
                                });
                            }
                        },
                        _ => {
                            let feature_name = format!("intrinsic {:?}", intrinsic);
                            return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                        }
                    },
                    _ => {
                        let feature_name = format!("calling {:?}", func);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                }
            }

            Instruction::IncrementRc { .. } => {
                // ignore
            }

            Instruction::DecrementRc { .. } => {
                // ignore
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

    fn get_type(&mut self, value_id: ValueId) -> Result<P2Type, Plonky2GenError> {
        let p2value: P2Value;
        let p2value_ref = match self.get(value_id) {
            Some(p2value) => p2value,
            None => {
                let value = self.dfg[value_id].clone();
                p2value = self.create_p2value(value)?;
                &p2value
            }
        };

        Ok(p2value_ref.typ)
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
                let message = format!("argument to get_integer is of type {:?}", p2value_ref.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let target = match p2value_ref.target {
            P2Target::IntTarget(target) => target,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_integer has non-integer target".to_owned(),
                })
            }
        };
        Ok((bit_size, target))
    }

    fn get_boolean(&mut self, value_id: ValueId) -> Result<BoolTarget, Plonky2GenError> {
        let p2value: P2Value;
        let p2value_ref = match self.get(value_id) {
            Some(p2value) => p2value,
            None => {
                let value = self.dfg[value_id].clone();
                p2value = self.create_p2value(value)?;
                &p2value
            }
        };

        let target = match p2value_ref.target {
            P2Target::BoolTarget(bool_target) => bool_target,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_boolean has non-boolean target".to_owned(),
                })
            }
        };
        Ok(target)
    }

    fn get_array(&mut self, value_id: ValueId) -> Result<(u32, Vec<Target>), Plonky2GenError> {
        let p2value = self.get(value_id).unwrap();
        let bit_size = match p2value.typ {
            P2Type::Array(bit_size) => bit_size,
            _ => {
                let message = format!("argument to get_array is of type {:?}", p2value.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let targets = match p2value.target {
            P2Target::ArrayTarget(ref targets) => targets.clone(),
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_array is not an array".to_owned(),
                })
            }
        };
        Ok((bit_size, targets))
    }

    fn get_array_element(
        &mut self,
        value_id: ValueId,
        index: usize,
    ) -> Result<(u32, Target), Plonky2GenError> {
        let p2value = self.get(value_id).unwrap();
        let bit_size = match p2value.typ {
            P2Type::Array(bit_size) => bit_size,
            _ => {
                let message = format!("argument to get_array_element is of type {:?}", p2value.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let target = match p2value.target {
            P2Target::ArrayTarget(ref targets) => targets[index].clone(),
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_array_element is not an array".to_owned(),
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
