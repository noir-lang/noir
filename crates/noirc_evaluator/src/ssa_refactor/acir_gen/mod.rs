//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use std::collections::HashMap;

use self::acir_ir::{
    acir_variable::{AcirContext, AcirType, AcirVar},
    errors::AcirGenError,
};
use super::{
    ir::{
        dfg::DataFlowGraph,
        function::RuntimeType,
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
        },
        map::Id,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use crate::brillig::{artifact::BrilligArtifact, Brillig};
use acvm::FieldElement;
use iter_extended::vecmap;

pub(crate) use acir_ir::generated_acir::GeneratedAcir;

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
#[derive(Default)]
struct Context {
    /// Maps SSA values to `AcirVar`.
    ///
    /// This is needed so that we only create a single
    /// AcirVar per SSA value. Before creating an `AcirVar`
    /// for an SSA value, we check this map. If an `AcirVar`
    /// already exists for this Value, we return the `AcirVar`.
    ssa_values: HashMap<Id<Value>, AcirValue>,

    /// The `AcirVar` that describes the condition belonging to the most recently invoked
    /// `SideEffectsEnabled` instruction.
    current_side_effects_enabled_var: Option<AcirVar>,

    /// Manages and builds the `AcirVar`s to which the converted SSA values refer.
    acir_context: AcirContext,
}

#[derive(Debug, Clone)]
pub(crate) enum AcirValue {
    Var(AcirVar, AcirType),
    Array(im::Vector<AcirValue>),
}

impl AcirValue {
    fn into_var(self) -> AcirVar {
        match self {
            AcirValue::Var(var, _) => var,
            AcirValue::Array(_) => panic!("Called AcirValue::into_var on an array"),
        }
    }

    fn flatten(self) -> Vec<(AcirVar, AcirType)> {
        match self {
            AcirValue::Var(var, typ) => vec![(var, typ)],
            AcirValue::Array(array) => array.into_iter().flat_map(AcirValue::flatten).collect(),
        }
    }
}

impl Ssa {
    pub(crate) fn into_acir(self, brillig: Brillig, allow_log_ops: bool) -> GeneratedAcir {
        let context = Context::default();
        context.convert_ssa(self, brillig, allow_log_ops)
    }
}

impl Context {
    /// Converts SSA into ACIR
    fn convert_ssa(mut self, ssa: Ssa, brillig: Brillig, allow_log_ops: bool) -> GeneratedAcir {
        let main_func = ssa.main();
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];

        self.convert_ssa_block_params(entry_block.parameters(), dfg);

        for instruction_id in entry_block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg, &ssa, &brillig, allow_log_ops);
        }

        self.convert_ssa_return(entry_block.terminator().unwrap(), dfg);

        self.acir_context.finish()
    }

    /// Adds and binds `AcirVar`s for each numeric block parameter or block parameter array element.
    fn convert_ssa_block_params(&mut self, params: &[ValueId], dfg: &DataFlowGraph) {
        for param_id in params {
            let typ = dfg.type_of_value(*param_id);
            let value = self.convert_ssa_block_param(&typ);
            self.ssa_values.insert(*param_id, value);
        }
    }

    fn convert_ssa_block_param(&mut self, param_type: &Type) -> AcirValue {
        match param_type {
            Type::Numeric(numeric_type) => {
                let typ = AcirType::new(*numeric_type);
                AcirValue::Var(self.add_numeric_input_var(numeric_type), typ)
            }
            Type::Array(element_types, length) => {
                let mut elements = im::Vector::new();

                for _ in 0..*length {
                    for element in element_types.iter() {
                        elements.push_back(self.convert_ssa_block_param(element));
                    }
                }

                AcirValue::Array(elements)
            }
            _ => unreachable!("ICE: Params to the program should only contains numbers and arrays"),
        }
    }

    /// Creates an `AcirVar` corresponding to a parameter witness to appears in the abi. A range
    /// constraint is added if the numeric type requires it.
    ///
    /// This function is used not only for adding numeric block parameters, but also for adding
    /// any array elements that belong to reference type block parameters.
    fn add_numeric_input_var(&mut self, numeric_type: &NumericType) -> AcirVar {
        let acir_var = self.acir_context.add_variable();
        if matches!(numeric_type, NumericType::Signed { .. } | NumericType::Unsigned { .. }) {
            self.acir_context
                .numeric_cast_var(acir_var, numeric_type)
                .expect("invalid range constraint was applied {numeric_type}");
        }
        acir_var
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        brillig: &Brillig,
        allow_log_ops: bool,
    ) {
        let instruction = &dfg[instruction_id];

        match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self
                    .convert_ssa_binary(binary, dfg)
                    .expect("add Result types to all methods so errors bubble up");
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Constrain(value_id) => {
                let constrain_condition = self.convert_numeric_value(*value_id, dfg);
                self.acir_context
                    .assert_eq_one(constrain_condition)
                    .expect("add Result types to all methods so errors bubble up");
            }
            Instruction::Cast(value_id, typ) => {
                let result_acir_var = self.convert_ssa_cast(value_id, typ, dfg);
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Call { func, arguments } => {
                let result_ids = dfg.instruction_results(instruction_id);
                match &dfg[*func] {
                    Value::Function(id) => {
                        let func = &ssa.functions[id];
                        match func.runtime() {
                            RuntimeType::Acir => unimplemented!(
                                "expected an intrinsic/brillig call, but found {func:?}. All ACIR methods should be inlined"
                            ),
                            RuntimeType::Brillig => {
                                let inputs:Vec<AcirVar> = arguments
                                    .iter()
                                    .flat_map(|arg| self.convert_value(*arg, dfg).flatten())
                                    .map(|(var, _typ)| var)
                                    .collect();
                                // Generate the brillig code of the function
                                let code = BrilligArtifact::default().link(&brillig[*id]);
                                let outputs = self.acir_context.brillig(code, inputs, result_ids.len());
                                for (result, output) in result_ids.iter().zip(outputs) {
                                    let result_acir_type = dfg.type_of_value(*result).into();
                                    self.ssa_values.insert(*result, AcirValue::Var(output, result_acir_type));
                                }
                            }
                        }
                    }
                    Value::Intrinsic(intrinsic) => {
                        let outputs = self.convert_ssa_intrinsic_call(
                            *intrinsic,
                            arguments,
                            dfg,
                            allow_log_ops,
                            result_ids,
                        );

                        // Issue #1438 causes this check to fail with intrinsics that return 0
                        // results but the ssa form instead creates 1 unit result value.
                        // assert_eq!(result_ids.len(), outputs.len());

                        for (result, output) in result_ids.iter().zip(outputs) {
                            self.ssa_values.insert(*result, output);
                        }
                    }
                    _ => unreachable!("expected calling a function"),
                }
            }
            Instruction::Not(value_id) => {
                let boolean_var = self.convert_numeric_value(*value_id, dfg);
                let result_acir_var = self.acir_context.not_var(boolean_var);
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                let result_acir_var = self
                    .convert_ssa_truncate(*value, *bit_size, *max_bit_size, dfg)
                    .expect("add Result types to all methods so errors bubble up");
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::EnableSideEffects { condition } => {
                let acir_var = self.convert_numeric_value(*condition, dfg);
                self.current_side_effects_enabled_var = Some(acir_var);
            }
            Instruction::ArrayGet { array, index } => {
                self.handle_array_operation(instruction_id, *array, *index, None, dfg);
            }
            Instruction::ArraySet { array, index, value } => {
                self.handle_array_operation(instruction_id, *array, *index, Some(*value), dfg);
            }
            Instruction::Allocate => {
                unreachable!("Expected all allocate instructions to be removed before acir_gen")
            }
            Instruction::Store { .. } => {
                unreachable!("Expected all store instructions to be removed before acir_gen")
            }
            Instruction::Load { .. } => {
                unreachable!("Expected all load instructions to be removed before acir_gen")
            }
        }
    }

    /// Handles an ArrayGet or ArraySet instruction.
    /// To set an index of the array (and create a new array in doing so), pass Some(value) for
    /// store_value. To just retrieve an index of the array, pass None for store_value.
    fn handle_array_operation(
        &mut self,
        instruction: InstructionId,
        array: ValueId,
        index: ValueId,
        store_value: Option<ValueId>,
        dfg: &DataFlowGraph,
    ) {
        let array = self.convert_array_value(array, dfg);
        let index = dfg
            .get_numeric_constant(index)
            .expect("Expected array index to be a known constant")
            .try_to_u64()
            .expect("Expected array index to fit into a u64") as usize;

        let value = match store_value {
            Some(store_value) => {
                let store_value = self.convert_value(store_value, dfg);
                AcirValue::Array(array.update(index, store_value))
            }
            None => array[index].clone(),
        };

        self.define_result(dfg, instruction, value);
    }

    /// Remember the result of an instruction returning a single value
    fn define_result(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirValue,
    ) {
        let result_ids = dfg.instruction_results(instruction);
        self.ssa_values.insert(result_ids[0], result);
    }

    /// Remember the result of instruction returning a single numeric value
    fn define_result_var(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirVar,
    ) {
        let result_ids = dfg.instruction_results(instruction);
        let typ = dfg.type_of_value(result_ids[0]).into();
        self.define_result(dfg, instruction, AcirValue::Var(result, typ));
    }

    /// Converts an SSA terminator's return values into their ACIR representations
    fn convert_ssa_return(&mut self, terminator: &TerminatorInstruction, dfg: &DataFlowGraph) {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => unreachable!("ICE: Program must have a singular return"),
        };

        // The return value may or may not be an array reference. Calling `flatten_value_list`
        // will expand the array if there is one.
        let return_acir_vars = self.flatten_value_list(return_values, dfg);

        for acir_var in return_acir_vars {
            self.acir_context.return_var(acir_var);
        }
    }

    /// Gets the cached `AcirVar` that was converted from the corresponding `ValueId`. If it does
    /// not already exist in the cache, a conversion is attempted and cached for simple values
    /// that require no further context such as numeric types - values requiring more context
    /// should have already been cached elsewhere.
    ///
    /// Conversion is assumed to have already been performed for instruction results and block
    /// parameters. This is because block parameters are converted before anything else, and
    /// because instructions results are converted when the corresponding instruction is
    /// encountered. (An instruction result cannot be referenced before the instruction occurs.)
    ///
    /// It is not safe to call this function on value ids that represent addresses. Instructions
    /// involving such values are evaluated via a separate path and stored in
    /// `ssa_value_to_array_address` instead.
    fn convert_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirValue {
        let value_id = dfg.resolve(value_id);
        let value = &dfg[value_id];
        if let Some(acir_value) = self.ssa_values.get(&value_id) {
            return acir_value.clone();
        }

        let acir_value = match value {
            Value::NumericConstant { constant, typ } => {
                AcirValue::Var(self.acir_context.add_constant(*constant), typ.into())
            }
            Value::Array { array, .. } => {
                let elements = array.iter().map(|element| self.convert_value(*element, dfg));
                AcirValue::Array(elements.collect())
            }
            Value::Intrinsic(..) => todo!(),
            Value::Function(..) => unreachable!("ICE: All functions should have been inlined"),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value:?}")
            }
        };
        self.ssa_values.insert(value_id, acir_value.clone());
        acir_value
    }

    fn convert_array_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> im::Vector<AcirValue> {
        match self.convert_value(value_id, dfg) {
            AcirValue::Var(acir_var, _) => panic!("Expected an array value, found: {acir_var:?}"),
            AcirValue::Array(array) => array,
        }
    }

    fn convert_numeric_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirVar {
        match self.convert_value(value_id, dfg) {
            AcirValue::Var(acir_var, _) => acir_var,
            AcirValue::Array(array) => panic!("Expected a numeric value, found: {array:?}"),
        }
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, AcirGenError> {
        let lhs = self.convert_numeric_value(binary.lhs, dfg);
        let rhs = self.convert_numeric_value(binary.rhs, dfg);

        let binary_type = self.type_of_binary_operation(binary, dfg);
        match &binary_type {
            Type::Numeric(NumericType::Unsigned { bit_size })
            | Type::Numeric(NumericType::Signed { bit_size }) => {
                // Conservative max bit size that is small enough such that two operands can be
                // multiplied and still fit within the field modulus. This is necessary for the
                // truncation technique: result % 2^bit_size to be valid.
                let max_integer_bit_size = FieldElement::max_num_bits() / 2;
                if *bit_size > max_integer_bit_size {
                    return Err(AcirGenError::UnsupportedIntegerSize {
                        num_bits: *bit_size,
                        max_num_bits: max_integer_bit_size,
                    });
                }
            }
            _ => {}
        }

        let binary_type = AcirType::from(binary_type);
        let bit_count = binary_type.bit_size();

        match binary.operator {
            BinaryOp::Add => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub => self.acir_context.sub_var(lhs, rhs),
            BinaryOp::Mul => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(lhs, rhs, binary_type),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => self.acir_context.less_than_var(
                lhs,
                rhs,
                bit_count,
                self.current_side_effects_enabled_var,
            ),
            BinaryOp::Shl => self.acir_context.shift_left_var(lhs, rhs, binary_type),
            BinaryOp::Shr => self.acir_context.shift_right_var(lhs, rhs, binary_type),
            BinaryOp::Xor => self.acir_context.xor_var(lhs, rhs, binary_type),
            BinaryOp::And => self.acir_context.and_var(lhs, rhs, binary_type),
            BinaryOp::Or => self.acir_context.or_var(lhs, rhs, binary_type),
            BinaryOp::Mod => self.acir_context.modulo_var(lhs, rhs, bit_count),
        }
    }

    /// Operands in a binary operation are checked to have the same type.
    ///
    /// In Noir, binary operands should have the same type due to the language
    /// semantics.
    ///
    /// There are some edge cases to consider:
    /// - Constants are not explicitly type casted, so we need to check for this and
    /// return the type of the other operand, if we have a constant.
    /// - 0 is not seen as `Field 0` but instead as `Unit 0`
    /// TODO: The latter seems like a bug, if we cannot differentiate between a function returning
    /// TODO nothing and a 0.
    ///
    /// TODO: This constant coercion should ideally be done in the type checker.
    fn type_of_binary_operation(&self, binary: &Binary, dfg: &DataFlowGraph) -> Type {
        let lhs_type = dfg.type_of_value(binary.lhs);
        let rhs_type = dfg.type_of_value(binary.rhs);

        match (lhs_type, rhs_type) {
            // Function type should not be possible, since all functions
            // have been inlined.
            (_, Type::Function) | (Type::Function, _) => {
                unreachable!("all functions should be inlined")
            }
            (_, Type::Reference) | (Type::Reference, _) => {
                unreachable!("References are invalid in binary operations")
            }
            (_, Type::Array(..)) | (Type::Array(..), _) => {
                unreachable!("Arrays are invalid in binary operations")
            }
            // If either side is a Field constant then, we coerce into the type
            // of the other operand
            (Type::Numeric(NumericType::NativeField), typ)
            | (typ, Type::Numeric(NumericType::NativeField)) => typ,
            // If either side is a numeric type, then we expect their types to be
            // the same.
            (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
                assert_eq!(
                    lhs_type, rhs_type,
                    "lhs and rhs types in a binary operation are always the same"
                );
                Type::Numeric(lhs_type)
            }
        }
    }

    /// Returns an `AcirVar` that is constrained to be
    fn convert_ssa_cast(&mut self, value_id: &ValueId, typ: &Type, dfg: &DataFlowGraph) -> AcirVar {
        let variable = self.convert_numeric_value(*value_id, dfg);

        match typ {
            Type::Numeric(numeric_type) => self
                .acir_context
                .numeric_cast_var(variable, numeric_type)
                .expect("invalid range constraint was applied {numeric_type}"),
            _ => unimplemented!("The cast operation is only valid for integers."),
        }
    }

    /// Returns an `AcirVar`that is constrained to be result of the truncation.
    fn convert_ssa_truncate(
        &mut self,
        value_id: ValueId,
        bit_size: u32,
        max_bit_size: u32,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, AcirGenError> {
        let mut var = self.convert_numeric_value(value_id, dfg);
        let truncation_target = match &dfg[value_id] {
            Value::Instruction { instruction, .. } => &dfg[*instruction],
            _ => unreachable!("ICE: Truncates are only ever applied to the result of a binary op"),
        };
        if matches!(truncation_target, Instruction::Binary(Binary { operator: BinaryOp::Sub, .. }))
        {
            // Subtractions must first have the integer modulus added before truncation can be
            // applied. This is done in order to prevent underflow.
            let integer_modulus =
                self.acir_context.add_constant(FieldElement::from(2_u128.pow(bit_size)));
            var = self.acir_context.add_var(var, integer_modulus)?;
        }

        self.acir_context.truncate_var(var, bit_size, max_bit_size)
    }

    /// Returns a vector of `AcirVar`s constrained to be result of the function call.
    ///
    /// The function being called is required to be intrinsic.
    fn convert_ssa_intrinsic_call(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        allow_log_ops: bool,
        result_ids: &[ValueId],
    ) -> Vec<AcirValue> {
        match intrinsic {
            Intrinsic::BlackBox(black_box) => {
                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));

                let vars = self
                    .acir_context
                    .black_box_function(black_box, inputs)
                    .expect("add Result types to all methods so errors bubble up");

                Self::convert_vars_to_values(vars, dfg, result_ids)
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var();
                let radix = self.convert_value(arguments[1], dfg).into_var();
                let limb_size = self.convert_value(arguments[2], dfg).into_var();
                let result_type = Self::array_element_type(dfg, result_ids[0]);

                self.acir_context
                    .radix_decompose(endian, field, radix, limb_size, result_type)
                    .expect("add Result types to all methods so errors bubble up")
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var();
                let bit_size = self.convert_value(arguments[1], dfg).into_var();
                let result_type = Self::array_element_type(dfg, result_ids[0]);

                self.acir_context
                    .bit_decompose(endian, field, bit_size, result_type)
                    .expect("add Result types to all methods so errors bubble up")
            }
            Intrinsic::Println => {
                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
                if allow_log_ops {
                    self.acir_context
                        .print(inputs)
                        .expect("add Result types to all methods so errors bubble up");
                }
                Vec::new()
            }
            _ => todo!("expected a black box function"),
        }
    }

    /// Given an array value, return the numerical type of its element.
    /// Panics if the given value is not an array or has a non-numeric element type.
    fn array_element_type(dfg: &DataFlowGraph, value: ValueId) -> AcirType {
        match dfg.type_of_value(value) {
            Type::Array(elements, _) => {
                assert_eq!(elements.len(), 1);
                (&elements[0]).into()
            }
            _ => unreachable!("Expected array type"),
        }
    }

    /// Maps an ssa value list, for which some values may be references to arrays, by inlining
    /// the `AcirVar`s corresponding to the contents of each array into the list of `AcirVar`s
    /// that correspond to other values.
    fn flatten_value_list(&mut self, arguments: &[ValueId], dfg: &DataFlowGraph) -> Vec<AcirVar> {
        let mut acir_vars = Vec::with_capacity(arguments.len());
        for value_id in arguments {
            let value = self.convert_value(*value_id, dfg);
            AcirContext::flatten_value(&mut acir_vars, value);
        }
        acir_vars
    }

    fn bit_count(&self, lhs: ValueId, dfg: &DataFlowGraph) -> u32 {
        match dfg.type_of_value(lhs) {
            Type::Numeric(NumericType::Signed { bit_size }) => bit_size,
            Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
            Type::Numeric(NumericType::NativeField) => FieldElement::max_num_bits(),
            _ => 0,
        }
    }

    /// Convert a Vec<AcirVar> into a Vec<AcirValue> using the given result ids.
    /// If the type of a result id is an array, several acirvars are collected into
    /// a single AcirValue::Array of the same length.
    fn convert_vars_to_values(
        vars: Vec<AcirVar>,
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Vec<AcirValue> {
        let mut vars = vars.into_iter();
        vecmap(result_ids, |result| {
            let result_type = dfg.type_of_value(*result);
            Self::convert_var_type_to_values(&result_type, &mut vars)
        })
    }

    /// Recursive helper for convert_vars_to_values.
    /// If the given result_type is an array of length N, this will create an AcirValue::Array with
    /// the first N elements of the given iterator. Otherwise, the result is a single
    /// AcirValue::Var wrapping the first element of the iterator.
    fn convert_var_type_to_values(
        result_type: &Type,
        vars: &mut impl Iterator<Item = AcirVar>,
    ) -> AcirValue {
        match result_type {
            Type::Array(elements, size) => {
                let mut element_values = im::Vector::new();
                for _ in 0..*size {
                    for element_type in elements.iter() {
                        let element = Self::convert_var_type_to_values(element_type, vars);
                        element_values.push_back(element);
                    }
                }
                AcirValue::Array(element_values)
            }
            typ => {
                let var = vars.next().unwrap();
                AcirValue::Var(var, typ.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use acvm::{
        acir::{
            circuit::Opcode,
            native_types::{Expression, Witness},
        },
        FieldElement,
    };

    use crate::{
        brillig::Brillig,
        ssa_refactor::{
            ir::{function::RuntimeType, map::Id, types::Type},
            ssa_builder::FunctionBuilder,
        },
    };

    use super::Context;

    #[test]
    fn returns_body_scoped_arrays() {
        // fn main {
        //   b0():
        //     return [Field 1]
        // }
        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);

        let one = builder.field_constant(FieldElement::one());

        let element_type = Rc::new(vec![Type::field()]);
        let array = builder.array_constant(im::Vector::unit(one), element_type);

        builder.terminate_with_return(vec![array]);

        let ssa = builder.finish();

        let context = Context::default();
        let acir = context.convert_ssa(ssa, Brillig::default(), false);

        let expected_opcodes =
            vec![Opcode::Arithmetic(&Expression::one() - &Expression::from(Witness(1)))];
        assert_eq!(acir.opcodes, expected_opcodes);
        assert_eq!(acir.return_witnesses, vec![Witness(1)]);
    }
}
