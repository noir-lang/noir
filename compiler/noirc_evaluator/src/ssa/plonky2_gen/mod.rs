mod asm_writer;
mod circuit;
mod config;
pub mod console_asm_writer;
mod div_generator;
mod intrinsics;

use super::{
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, InstructionId, Intrinsic},
    },
    ssa_gen::Ssa,
};
use acvm::{acir::BlackBoxFunc, AcirField, FieldElement};
use asm_writer::AsmWriter;
pub use circuit::Plonky2Circuit;
use div_generator::add_div_mod;
use noirc_frontend::{ast::Visibility, hir_def::function::FunctionSignature};
use plonky2::{
    field::types::Field, iop::target::BoolTarget, iop::target::Target,
    plonk::circuit_data::CircuitConfig,
};
use std::collections::HashMap;

use self::config::{P2Builder, P2Config, P2Field};
use self::intrinsics::make_sha256_circuit;

use crate::errors::{Plonky2GenError, RuntimeError};
use crate::ssa::ir::{
    dfg::CallStack,
    instruction::Instruction,
    types::NumericType,
    types::Type,
    value::{Value, ValueId},
};

#[derive(Debug, Eq, PartialEq)]
struct P2Value {
    target: P2Target,
    typ: P2Type,
}

impl P2Value {
    fn get_target(&self) -> Result<Target, Plonky2GenError> {
        self.target.get_target()
    }

    fn get_integer_target(&self) -> Result<Target, Plonky2GenError> {
        match self.target {
            P2Target::IntTarget(target) => Ok(target),
            _ => {
                let message = "get_integer_target called on non-int value".to_owned();
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn get_boolean_target(&self) -> Result<BoolTarget, Plonky2GenError> {
        match self.target {
            P2Target::BoolTarget(bool_target) => Ok(bool_target),
            _ => {
                let message = format!("get_boolean_target called on non-bool {:?}", self);
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn get_array_targets(&self) -> Result<Vec<P2Target>, Plonky2GenError> {
        match self.target {
            P2Target::ArrayTarget(ref targets) => Ok(targets.clone()),
            _ => {
                let message = format!("get_array_targets called on non-array {:?}", self);
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn get_struct_targets(&self) -> Result<Vec<P2Target>, Plonky2GenError> {
        match self.target {
            P2Target::StructTarget(ref targets) => Ok(targets.clone()),
            _ => {
                let message = format!("get_struct_targets called on non-struct {:?}", self);
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn make_integer(p2type: P2Type, target: Target) -> Result<P2Value, Plonky2GenError> {
        match p2type {
            P2Type::Integer(_) => Ok(P2Value { target: P2Target::IntTarget(target), typ: p2type }),
            P2Type::Field => Ok(P2Value::make_field(target)),
            _ => {
                let message = format!("make_integer called for type {:?}", p2type);
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn make_boolean(target: BoolTarget) -> P2Value {
        P2Value { target: P2Target::BoolTarget(target), typ: P2Type::Boolean }
    }

    fn make_field(target: Target) -> P2Value {
        P2Value { target: P2Target::IntTarget(target), typ: P2Type::Field }
    }

    fn make_array(element_type: P2Type, targets: Vec<P2Target>) -> P2Value {
        P2Value {
            target: P2Target::ArrayTarget(targets.clone()),
            typ: P2Type::Array(Box::new(element_type), targets.len()),
        }
    }

    fn make_struct(field_types: Vec<P2Type>, targets: Vec<P2Target>) -> P2Value {
        P2Value {
            target: P2Target::StructTarget(targets.clone()),
            typ: P2Type::Struct(field_types),
        }
    }

    /// Creates an undefined PLONKY2 value of the given type, which includes adding targets to the given
    /// builder.
    fn create_empty(builder: &mut P2Builder, p2type: P2Type) -> P2Value {
        match p2type.clone() {
            P2Type::Field => {
                P2Value { target: P2Target::IntTarget(builder.add_virtual_target()), typ: p2type }
            }
            P2Type::Integer(_) => {
                P2Value { target: P2Target::IntTarget(builder.add_virtual_target()), typ: p2type }
            }
            P2Type::Boolean => P2Value {
                target: P2Target::BoolTarget(builder.add_virtual_bool_target_safe()),
                typ: p2type,
            },
            P2Type::Array(element_type, array_size) => {
                let mut p2targets = Vec::new();
                for _ in 0..array_size {
                    p2targets.push(P2Value::create_empty(builder, *element_type.clone()).target);
                }
                P2Value { target: P2Target::ArrayTarget(p2targets), typ: p2type }
            }
            P2Type::Struct(field_types) => {
                let mut p2targets = Vec::new();
                for field_type in field_types {
                    p2targets.push(P2Value::create_empty(builder, field_type.clone()).target);
                }
                P2Value { target: P2Target::StructTarget(p2targets), typ: p2type }
            }
        }
    }

    fn create_simple_constant(
        asm_writer: &mut impl AsmWriter,
        p2type: P2Type,
        constant: FieldElement,
    ) -> Result<P2Value, Plonky2GenError> {
        match p2type.clone() {
            P2Type::Field => {
                let target = asm_writer.constant(noir_to_plonky2_field(constant));
                Ok(P2Value { target: P2Target::IntTarget(target), typ: p2type })
            }
            P2Type::Integer(_) => {
                let target = asm_writer.constant(noir_to_plonky2_field(constant));
                Ok(P2Value { target: P2Target::IntTarget(target), typ: p2type })
            }
            P2Type::Boolean => {
                let target = asm_writer.constant_bool(constant.to_u128() != 0);
                Ok(P2Value { target: P2Target::BoolTarget(target), typ: p2type })
            }
            _ => {
                let message =
                    format!("create_simple_constant called with an argument of type {:?}", p2type);
                Err(Plonky2GenError::ICE { message })
            }
        }
    }

    fn clone(&self) -> Result<P2Value, Plonky2GenError> {
        Ok(match self.typ.clone() {
            P2Type::Integer(_) => {
                P2Value::make_integer(self.typ.clone(), self.get_integer_target()?.clone())?
            }
            P2Type::Boolean => P2Value::make_boolean(self.get_boolean_target()?.clone()),
            P2Type::Field => P2Value::make_field(self.get_integer_target()?.clone()),
            P2Type::Array(typ, _) => P2Value::make_array(*typ, self.get_array_targets()?),
            P2Type::Struct(types) => P2Value::make_struct(types, self.get_struct_targets()?),
        })
    }
}

// TODO(stanm): be more precise here.
const FIELD_BIT_SIZE: u32 = 254;

#[derive(Debug, Clone, Eq, PartialEq)]
enum P2Type {
    Boolean,
    Integer(u32),
    Array(Box<P2Type>, usize),
    Struct(Vec<P2Type>),
    Field,
}

impl P2Type {
    fn from_noir_type(typ: Type) -> Result<P2Type, Plonky2GenError> {
        Ok(match typ {
            Type::Numeric(numeric_type) => match numeric_type {
                NumericType::NativeField => P2Type::Field,
                NumericType::Unsigned { bit_size } => {
                    if bit_size == 1 {
                        P2Type::Boolean
                    } else {
                        P2Type::Integer(bit_size)
                    }
                }
                _ => {
                    let feature_name = format!("the {numeric_type} type");
                    return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                }
            },
            Type::Array(composite_type, array_size) => {
                if array_size == 1 {
                    P2Type::from_noir_types((*composite_type).clone())?
                } else {
                    P2Type::Array(
                        Box::new(P2Type::from_noir_types((*composite_type).clone())?),
                        array_size,
                    )
                }
            }
            _ => {
                let feature_name = format!("the {typ} type");
                return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
            }
        })
    }

    fn from_noir_types(types: Vec<Type>) -> Result<P2Type, Plonky2GenError> {
        if types.len() == 1 {
            P2Type::from_noir_type(types[0].clone())
        } else {
            let mut field_types = Vec::new();
            for typ in types {
                field_types.push(P2Type::from_noir_type(typ)?);
            }
            Ok(P2Type::Struct(field_types))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum P2Target {
    IntTarget(Target),
    BoolTarget(BoolTarget),
    ArrayTarget(Vec<P2Target>),
    StructTarget(Vec<P2Target>),
}

impl P2Target {
    fn get_target(&self) -> Result<Target, Plonky2GenError> {
        Ok(match &self {
            P2Target::IntTarget(target) => target.clone(),
            P2Target::BoolTarget(bool_target) => bool_target.target,
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "get_target called on a non-int, non-bool value".to_owned(),
                })
            }
        })
    }

    /// Extends the given list with the Noir targets wrapped by this p2target: if this p2target is an
    /// array, there would be multiple targets wrapped.
    fn extend_parameter_list(&self, parameters: &mut Vec<Target>) -> Result<(), Plonky2GenError> {
        Ok(match &self {
            P2Target::ArrayTarget(ref targets) => {
                for target in targets {
                    let _ = target.extend_parameter_list(parameters)?;
                }
            }
            P2Target::StructTarget(ref targets) => {
                for target in targets {
                    let _ = target.extend_parameter_list(parameters)?;
                }
            }
            _ => parameters.push(self.get_target()?),
        })
    }

    fn clone(&self) -> P2Target {
        match &self {
            P2Target::IntTarget(target) => P2Target::IntTarget(target.clone()),
            P2Target::BoolTarget(bool_target) => P2Target::BoolTarget(bool_target.clone()),
            P2Target::ArrayTarget(targets) => P2Target::ArrayTarget(targets.clone()),
            P2Target::StructTarget(targets) => P2Target::StructTarget(targets.clone()),
        }
    }
}

pub(crate) struct Builder<TAsmWriter: AsmWriter> {
    asm_writer: TAsmWriter,
    translation: HashMap<ValueId, P2Value>,
    dfg: DataFlowGraph,
    show_plonky2: bool,
}

impl<TAsmWriter> Builder<TAsmWriter>
where
    TAsmWriter: AsmWriter,
{
    pub(crate) fn new(show_plonky2: bool) -> Builder<TAsmWriter> {
        let config = CircuitConfig::standard_recursion_config();
        Builder::<TAsmWriter> {
            asm_writer: TAsmWriter::new(P2Builder::new(config), show_plonky2),
            translation: HashMap::new(),
            dfg: DataFlowGraph::default(),
            show_plonky2,
        }
    }

    fn get_builder(&self) -> &P2Builder {
        self.asm_writer.get_builder()
    }
    fn get_mut_builder(&mut self) -> &mut P2Builder {
        self.asm_writer.get_mut_builder()
    }

    pub(crate) fn build(
        mut self,
        ssa: Ssa,
        parameter_names: Vec<String>,
        main_function_signature: FunctionSignature,
    ) -> Result<Plonky2Circuit, RuntimeError> {
        let main_function =
            ssa.functions.into_values().find(|value| value.name() == "main").unwrap();
        let entry_block_id = main_function.entry_block();
        self.dfg = main_function.dfg;
        let entry_block = self.dfg[entry_block_id].clone();

        let mut parameters = Vec::new();
        for value_id in entry_block.parameters().iter() {
            self.add_parameter(*value_id)?;
            let p2value = self.get(*value_id).unwrap();
            match p2value.target.extend_parameter_list(&mut parameters) {
                Ok(_) => {}
                Err(error) => {
                    return Err(
                        error.into_runtime_error("parameter list".to_owned(), CallStack::new())
                    );
                }
            }
        }
        for instruction_id in entry_block.instructions() {
            match self.add_instruction(*instruction_id) {
                Err(error) => {
                    let instruction = format!("{:?}", self.dfg[*instruction_id].clone());
                    return Err(error.into_runtime_error(
                        instruction,
                        self.dfg.get_call_stack(*instruction_id),
                    ));
                }
                Ok(_) => (),
            }
        }
        let mut next_param_idx: usize = 0;
        for (_, typ, vis) in main_function_signature.0 {
            let fields_for_param = typ.field_count() as usize;
            if vis == Visibility::Public {
                self.asm_writer.register_public_inputs(
                    &parameters[next_param_idx..next_param_idx+fields_for_param]
                );
            }
            next_param_idx += fields_for_param;
        }
        let data = self.asm_writer.move_builder().build::<P2Config>();
        // println!("stanm: data={:?}", data);
        Ok(Plonky2Circuit { data, parameters, parameter_names })
    }

    fn add_parameter(&mut self, value_id: ValueId) -> Result<(), Plonky2GenError> {
        let value = self.dfg[value_id].clone();
        let p2value = match value {
            Value::Param { block: _, position: _, typ } => {
                let p2type = P2Type::from_noir_type(typ)?;
                P2Value::create_empty(&mut self.get_mut_builder(), p2type)
            }
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
        p2builder_op: fn(&mut TAsmWriter, Target, Target) -> Target,
    ) -> Result<P2Value, Plonky2GenError> {
        let (type_a, target_a) = self.get_integer(lhs)?;
        let (type_b, target_b) = self.get_integer(rhs)?;
        if type_a != type_b {
            let message = format!("mismatching arg types: {:?} and {:?}", type_a, type_b);
            return Err(Plonky2GenError::ICE { message });
        }

        let target = p2builder_op(&mut self.asm_writer, target_a, target_b);

        P2Value::make_integer(type_a, target)
    }

    fn multi_convert_boolean_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut TAsmWriter, BoolTarget, BoolTarget) -> BoolTarget,
        opname: &str,
    ) -> Result<P2Value, Plonky2GenError> {
        let typ = self.get_type(lhs)?;
        match typ {
            P2Type::Boolean => self.convert_boolean_op(lhs, rhs, p2builder_op),
            P2Type::Integer(_) => self.convert_bitwise_logical_op(lhs, rhs, p2builder_op),
            P2Type::Field => self.convert_bitwise_logical_op(lhs, rhs, p2builder_op),
            _ => {
                let feature_name = format!("{:?} instruction on {:?}", opname, typ);
                return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
            }
        }
    }

    fn multi_convert_integer_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        normal_op: fn(&mut TAsmWriter, Target, Target) -> Target,
        boolean_op: fn(&mut TAsmWriter, BoolTarget, BoolTarget) -> BoolTarget,
        opname: &str,
    ) -> Result<P2Value, Plonky2GenError> {
        let typ = self.get_type(lhs)?;
        match typ {
            P2Type::Boolean => self.convert_boolean_op(lhs, rhs, boolean_op),
            P2Type::Integer(_) => self.convert_integer_op(lhs, rhs, normal_op),
            P2Type::Field => self.convert_integer_op(lhs, rhs, normal_op),
            _ => {
                let feature_name = format!("{:?} instruction on {:?}", opname, typ);
                return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
            }
        }
    }

    /// Converts from ssa::ir::instruction::BinaryOp to the equivalent P2Builder instruction, when
    /// such conversion is straightforward and the arguments are booleans.
    fn convert_boolean_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        p2builder_op: fn(&mut TAsmWriter, BoolTarget, BoolTarget) -> BoolTarget,
    ) -> Result<P2Value, Plonky2GenError> {
        let target_a = self.get_boolean(lhs)?;
        let target_b = self.get_boolean(rhs)?;

        let target = p2builder_op(&mut self.asm_writer, target_a, target_b);
        Ok(P2Value::make_boolean(target))
    }

    fn convert_bitwise_logical_op(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        single_bit_op: fn(&mut TAsmWriter, BoolTarget, BoolTarget) -> BoolTarget,
    ) -> Result<P2Value, Plonky2GenError> {
        let (type_a, target_a) = self.get_integer(lhs)?;
        let (type_b, target_b) = self.get_integer(rhs)?;
        assert!(type_a == type_b);
        let bit_size = usize::try_from(match type_a {
            P2Type::Integer(bit_size) => bit_size,
            P2Type::Field => FIELD_BIT_SIZE,
            _ => {
                let message =
                    format!("bitwise logical op invoked on arguments of type {:?}", type_a);
                return Err(Plonky2GenError::ICE { message });
            }
        })
        .unwrap();

        let a_bits = self.asm_writer.split_le(target_a, bit_size);
        let b_bits = self.asm_writer.split_le(target_b, bit_size);

        let mut result_bits = Vec::new();
        for (i, (a_bit, b_bit)) in a_bits.iter().zip(b_bits).enumerate() {
            let result_bit = single_bit_op(&mut self.asm_writer, *a_bit, b_bit);

            let zero = self.asm_writer.zero();
            let one = self.asm_writer.one();
            let two = self.asm_writer.two();
            let result_power_of_two = if i > 0 {
                let bit = self.asm_writer._if(result_bit, two, zero);
                self.asm_writer.exp_u64(bit, u64::try_from(i).unwrap())
            } else {
                self.asm_writer._if(result_bit, one, zero)
            };
            result_bits.push(result_power_of_two);
        }

        let target = self.asm_writer.add_many(result_bits);

        P2Value::make_integer(type_a, target)
    }

    fn perform_sha256(
        &mut self,
        argument: (P2Type, Vec<P2Target>),
        destination: ValueId,
    ) -> Result<(), Plonky2GenError> {
        /*
        * TODO(stanm)
        match argument.0 {
            P2Type::Integer(_) => {}
            _ => {
                let message = format!("wrong type of argument to perform_sha256: {:?}", argument.0);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        */
        let msg_len = u64::try_from(argument.1.len()).unwrap() * 8;
        let sha256targets = make_sha256_circuit(&mut self.get_mut_builder(), msg_len);
        let mut j = 0;
        for target in argument.1 {
            let split_arg = self.asm_writer.split_le(target.get_target()?, 8);
            for arg_bit in split_arg.iter().rev() {
                self.asm_writer.connect(arg_bit.target, sha256targets.message[j].target);
                j += 1;
            }
        }
        j = 0;
        let mut result = Vec::new();
        while j < 256 {
            result.push(P2Target::IntTarget(
                self.asm_writer.le_sum(sha256targets.digest[j..j + 8].iter().rev()),
            ));
            j += 8;
        }
        let p2value = P2Value::make_array(P2Type::Integer(8), result);
        self.set(destination, p2value);
        Ok(())
    }

    fn add_instruction(&mut self, instruction_id: InstructionId) -> Result<(), Plonky2GenError> {
        let instruction = self.dfg[instruction_id].clone();

        match instruction {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let p2value = match operator {
                    super::ir::instruction::BinaryOp::Mul => self.multi_convert_integer_op(
                        lhs,
                        rhs,
                        TAsmWriter::mul,
                        TAsmWriter::and,
                        "Mul",
                    ),

                    super::ir::instruction::BinaryOp::Div => {
                        self.convert_integer_op(lhs, rhs, |builder, t1, t2| {
                            add_div_mod(builder, t1, t2).0
                        })
                    }

                    super::ir::instruction::BinaryOp::Mod => {
                        self.convert_integer_op(lhs, rhs, |asm_writer, t1, t2| {
                            add_div_mod(asm_writer, t1, t2).1
                        })
                    }

                    super::ir::instruction::BinaryOp::Add => self.multi_convert_integer_op(
                        lhs,
                        rhs,
                        TAsmWriter::add,
                        TAsmWriter::or,
                        "Add",
                    ),

                    super::ir::instruction::BinaryOp::Sub => {
                        self.convert_integer_op(lhs, rhs, TAsmWriter::sub)
                    }

                    super::ir::instruction::BinaryOp::Eq => {
                        let target_a = self.get_target(lhs)?;
                        let target_b = self.get_target(rhs)?;
                        let target = self.asm_writer.is_equal(target_a, target_b);
                        Ok(P2Value::make_boolean(target))
                    }

                    super::ir::instruction::BinaryOp::Lt => {
                        let (bit_size_a, target_a) = self.get_integer(lhs)?;
                        let (bit_size_b, target_b) = self.get_integer(rhs)?;
                        assert!(bit_size_a == bit_size_b);

                        let div = add_div_mod(&mut self.asm_writer, target_a, target_b).0;
                        let zero = self.asm_writer.zero();
                        let target = self.asm_writer.is_equal(div, zero);
                        Ok(P2Value::make_boolean(target))
                    }

                    super::ir::instruction::BinaryOp::Xor => {
                        fn one_bit_xor(
                            asm_writer: &mut impl AsmWriter,
                            lhs: BoolTarget,
                            rhs: BoolTarget,
                        ) -> BoolTarget {
                            let not_lhs = asm_writer.not(lhs);
                            let not_rhs = asm_writer.not(rhs);
                            let c = asm_writer.and(lhs, not_rhs);
                            let d = asm_writer.and(not_lhs, rhs);
                            asm_writer.or(c, d)
                        }

                        self.multi_convert_boolean_op(lhs, rhs, one_bit_xor, "Xor")
                    }

                    super::ir::instruction::BinaryOp::And => {
                        self.multi_convert_boolean_op(lhs, rhs, TAsmWriter::and, "And")
                    }

                    super::ir::instruction::BinaryOp::Or => {
                        self.multi_convert_boolean_op(lhs, rhs, TAsmWriter::or, "Or")
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

            Instruction::Not(argument) => {
                let typ = self.get_type(argument)?;
                match typ {
                    P2Type::Boolean => {
                        let target = self.get_boolean(argument)?;
                        let target = self.asm_writer.not(target);
                        let p2value = P2Value::make_boolean(target);

                        let destinations: Vec<_> =
                            self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                        assert!(destinations.len() == 1);
                        self.set(destinations[0], p2value);
                    }
                    _ => {
                        let feature_name = format!("Not instruction on {:?}", typ);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                }
            }

            Instruction::Constrain(lhs, rhs, _) => {
                let a = self.get_target(lhs)?;
                let b = self.get_target(rhs)?;
                self.asm_writer.connect(a, b);
            }

            Instruction::RangeCheck { value, max_bit_size, assert_message: _ } => {
                let x = self.get_target(value)?;
                self.get_mut_builder().range_check(x, usize::try_from(max_bit_size).unwrap());
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
                let (p2type, p2target) = self.get_array_element(array, num_index)?;

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], P2Value { target: p2target, typ: p2type });
            }

            Instruction::ArraySet { array, index, value, mutable } => {
                let index = self.dfg[index].clone();
                let num_index = match index {
                    Value::NumericConstant { constant, .. } => constant.to_u128() as usize,
                    _ => {
                        let feature_name = format!("indexing array with an {:?}", index);
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };
                let (target_type, p2targets) = self.get_array(array)?;
                let p2value = match self.get(value) {
                    Some(p2value) => p2value.clone()?,
                    None => {
                        let value = self.dfg[value].clone();
                        self.create_p2value(value)?.clone()?
                    }
                };
                assert!(target_type == p2value.typ);

                let mut new_values = Vec::new();
                for i in 0..p2targets.len() {
                    new_values.push(P2Value::create_empty(&mut self.get_mut_builder(), target_type.clone()));
                    if i == num_index {
                        self.asm_writer.connect(p2value.get_target()?, new_values[i].get_target()?);
                    } else {
                        self.asm_writer
                            .connect(p2targets[i].get_target()?, new_values[i].get_target()?);
                    }
                }

                if mutable {
                    // It's hard to test this, so leaving it as a potential bug at the moment.
                    // self.set_array_element(array, num_index, new_value)?;
                }
                let mut new_targets = Vec::new();
                for p2value in new_values {
                    new_targets.push(p2value.target);
                }
                let new_array = P2Value::make_array(target_type.clone(), new_targets);

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], new_array);
            }

            Instruction::Call { func, arguments } => {
                let func = self.dfg[func].clone();

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

                                assert!(arguments.len() == 1);
                                let argument = self.get_array(arguments[0])?;
                                let _ = self.perform_sha256(argument, destinations[0])?;
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

            Instruction::EnableSideEffects { .. } => {
                // ignore
            }

            Instruction::IncrementRc { .. } => {
                // ignore
            }

            Instruction::DecrementRc { .. } => {
                // ignore
            }

            Instruction::Truncate { value, bit_size, max_bit_size } => {
                let p2value = self.get(value).unwrap();
                let target = p2value.get_target()?;
                let typ = p2value.typ.clone();
                let mut bits =
                    self.asm_writer.split_le(target, usize::try_from(max_bit_size).unwrap());
                bits.truncate(usize::try_from(bit_size).unwrap());
                let result = self.asm_writer.le_sum(bits.iter());
                // Note(stanm): truncate does not change the type of the input; it creates a value of the
                // same type, that should then be passed to Cast.
                let p2value = P2Value::make_integer(typ, result)?;

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], p2value);
            }

            Instruction::Cast(value_id, typ) => {
                // TODO(stanm): Add check that value is already truncated (if bit_size <
                // old_bit_size) for added safety.
                let p2value = self.get(value_id).unwrap();
                let target = p2value.get_target()?;
                let bit_size = match typ {
                    Type::Numeric(numeric_type) => match numeric_type {
                        NumericType::Unsigned { bit_size } => bit_size,
                        _ => {
                            let feature_name = format!("cast to {numeric_type}");
                            return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                        }
                    },
                    _ => {
                        let feature_name = format!("cast to {typ}");
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                };
                let new_target = self.get_mut_builder().add_virtual_target();
                self.asm_writer.connect(target, new_target);

                let p2value = P2Value::make_integer(P2Type::Integer(bit_size), new_target)?;

                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction_id).iter().cloned().collect();
                assert!(destinations.len() == 1);
                self.set(destinations[0], p2value);
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

    fn create_p2value(&mut self, value: Value) -> Result<P2Value, Plonky2GenError> {
        match value.clone() {
            Value::Param { typ, .. } => {
                let p2type = P2Type::from_noir_type(typ)?;
                Ok(P2Value::create_empty(&mut self.get_mut_builder(), p2type))
            }
            Value::NumericConstant { constant, typ } => {
                let p2type = P2Type::from_noir_type(typ)?;
                P2Value::create_simple_constant(&mut self.asm_writer, p2type, constant)
            }
            Value::Array { array, typ } => {
                assert!(array.len() > 0, "empty array literal");
                let element_type = P2Type::from_noir_type(typ)?;
                let mut targets = Vec::new();
                for element in array {
                    let p2value: P2Value;
                    let p2value_ref = match self.get(element) {
                        Some(p2value) => p2value,
                        None => {
                            let element = self.dfg.resolve(element);
                            let element_value = self.dfg[element].clone();
                            p2value = self.create_p2value(element_value)?;
                            &p2value
                        }
                    };
                    let actual_element_type = p2value_ref.typ.clone();
                    if element_type.clone() != actual_element_type {
                        let feature_name = format!(
                            "array elements of different types ({:?} and {:?}); array={:?}",
                            element_type, actual_element_type, value
                        );
                        return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                    }
                    targets.push(p2value_ref.target.clone());
                }

                Ok(P2Value::make_array(element_type.clone(), targets))
            }
            Value::Instruction { instruction, .. } => {
                let destinations: Vec<_> =
                    self.dfg.instruction_results(instruction).iter().cloned().collect();
                assert!(destinations.len() == 1);

                self.get(destinations[0]).unwrap().clone()
            }
            _ => Err(Plonky2GenError::ICE {
                message: format!("create_p2value passed a value that is {:?}", value),
            }),
        }
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

        Ok(p2value_ref.typ.clone())
    }

    fn get_integer(&mut self, value_id: ValueId) -> Result<(P2Type, Target), Plonky2GenError> {
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
            P2Target::IntTarget(target) => target,
            _ => {
                let message = format!(
                    "argument to get_integer has non-integer target {:?}",
                    p2value_ref.target
                );
                return Err(Plonky2GenError::ICE { message });
            }
        };
        Ok((p2value_ref.typ.clone(), target))
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

    fn get_array(&mut self, value_id: ValueId) -> Result<(P2Type, Vec<P2Target>), Plonky2GenError> {
        let p2value: P2Value;
        let p2value_ref = match self.get(value_id) {
            Some(p2value) => p2value,
            None => {
                let value = self.dfg[value_id].clone();
                p2value = self.create_p2value(value)?;
                &p2value
            }
        };
        let p2type = match p2value_ref.typ.clone() {
            P2Type::Array(p2type, _) => p2type,
            _ => {
                let message = format!("argument to get_array is of type {:?}", p2value_ref.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let targets = match p2value_ref.target {
            P2Target::ArrayTarget(ref targets) => targets.clone(),
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_array is not an array".to_owned(),
                })
            }
        };
        Ok((*p2type, targets))
    }

    fn get_array_element(
        &mut self,
        value_id: ValueId,
        index: usize,
    ) -> Result<(P2Type, P2Target), Plonky2GenError> {
        let p2value = self.get(value_id).unwrap();
        let element_type = match p2value.typ.clone() {
            P2Type::Array(p2type, _) => *p2type,
            _ => {
                let message = format!("argument to get_array_element is of type {:?}", p2value.typ);
                return Err(Plonky2GenError::ICE { message });
            }
        };
        let result = match p2value.target {
            P2Target::ArrayTarget(ref targets) => {
                match element_type.clone() {
                    P2Type::Struct(field_types) => {
                        let array_index = index / field_types.len();
                        let field_index = index % field_types.len();
                        match targets[array_index].clone() {
                            P2Target::StructTarget(fields) => {
                                (field_types[field_index].clone(), fields[field_index].clone())
                            }
                            _ => {
                                let message = format!(
                                    "Array element {:?} does not match type {:?}",
                                    targets[array_index], element_type
                                );
                                return Err(Plonky2GenError::ICE { message });
                            }
                        }
                    }
                    // TODO(stanm): arrays too like structs
                    _ => (element_type, targets[index].clone()),
                }
            }
            _ => {
                return Err(Plonky2GenError::ICE {
                    message: "argument to get_array_element is not an array".to_owned(),
                })
            }
        };
        Ok(result)
    }

    /// Get the PLONKY2 target of a value, regardless of whether its type is Integer or Boolean.
    fn get_target(&mut self, value_id: ValueId) -> Result<Target, Plonky2GenError> {
        let p2value: P2Value;
        let p2value_ref = match self.get(value_id) {
            Some(p2value) => p2value,
            None => {
                let value = self.dfg[value_id].clone();
                p2value = self.create_p2value(value)?;
                &p2value
            }
        };
        p2value_ref.get_target()
    }
}

pub(crate) fn noir_to_plonky2_field(field: FieldElement) -> P2Field {
    // TODO(plonky2): Noir doesn't support the Goldilock field. FieldElement is 254 bit, so if the
    // user enters a large integer this will fail.
    //
    // TODO(plonky2): Consider negative numbers.
    P2Field::from_canonical_u64(field.to_u128() as u64)
}
