//! This file holds the pass to convert from Noir's SSA IR to ACIR.
mod acir_ir;

use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::RangeInclusive;

use self::acir_ir::acir_variable::{AcirContext, AcirType, AcirVar};
use super::ir::dfg::CallStack;
use super::{
    ir::{
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
        },
        map::Id,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use crate::brillig::brillig_ir::artifact::GeneratedBrillig;
use crate::brillig::brillig_ir::BrilligContext;
use crate::brillig::{brillig_gen::brillig_fn::FunctionContext as BrilligFunctionContext, Brillig};
use crate::errors::{InternalError, RuntimeError};
pub(crate) use acir_ir::generated_acir::GeneratedAcir;
use acvm::{
    acir::{circuit::opcodes::BlockId, native_types::Expression},
    FieldElement,
};
use fxhash::FxHashMap as HashMap;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Distinctness;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
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
    current_side_effects_enabled_var: AcirVar,

    /// Manages and builds the `AcirVar`s to which the converted SSA values refer.
    acir_context: AcirContext,

    /// Track initialized acir dynamic arrays
    ///
    /// An acir array must start with a MemoryInit ACIR opcodes
    /// and then have MemoryOp opcodes
    /// This set is used to ensure that a MemoryOp opcode is only pushed to the circuit
    /// if there is already a MemoryInit opcode.
    initialized_arrays: HashSet<BlockId>,

    /// Maps SSA values to BlockId
    /// A BlockId is an ACIR structure which identifies a memory block
    /// Each acir memory block corresponds to a different SSA array.
    memory_blocks: HashMap<Id<Value>, BlockId>,

    /// Number of the next BlockId, it is used to construct
    /// a new BlockId
    max_block_id: u32,
}

#[derive(Clone)]
pub(crate) struct AcirDynamicArray {
    /// Identification for the Acir dynamic array
    /// This is essentially a ACIR pointer to the array
    block_id: BlockId,
    /// Length of the array
    len: usize,
}
impl Debug for AcirDynamicArray {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "id: {}, len: {}", self.block_id.0, self.len)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AcirValue {
    Var(AcirVar, AcirType),
    Array(im::Vector<AcirValue>),
    DynamicArray(AcirDynamicArray),
}

impl AcirValue {
    fn into_var(self) -> Result<AcirVar, InternalError> {
        match self {
            AcirValue::Var(var, _) => Ok(var),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => Err(InternalError::General {
                message: "Called AcirValue::into_var on an array".to_string(),
                call_stack: CallStack::new(),
            }),
        }
    }

    fn flatten(self) -> Vec<(AcirVar, AcirType)> {
        match self {
            AcirValue::Var(var, typ) => vec![(var, typ)],
            AcirValue::Array(array) => array.into_iter().flat_map(AcirValue::flatten).collect(),
            AcirValue::DynamicArray(_) => unimplemented!("Cannot flatten a dynamic array"),
        }
    }
}

impl Ssa {
    pub(crate) fn into_acir(
        self,
        brillig: Brillig,
        abi_distinctness: Distinctness,
        last_array_uses: &HashMap<ValueId, InstructionId>,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let context = Context::new();
        let mut generated_acir = context.convert_ssa(self, brillig, last_array_uses)?;

        match abi_distinctness {
            Distinctness::Distinct => {
                // Create a witness for each return witness we have
                // to guarantee that the return witnesses are distinct
                let distinct_return_witness: Vec<_> = generated_acir
                    .return_witnesses
                    .clone()
                    .into_iter()
                    .map(|return_witness| {
                        generated_acir
                            .create_witness_for_expression(&Expression::from(return_witness))
                    })
                    .collect();

                generated_acir.return_witnesses = distinct_return_witness;
                Ok(generated_acir)
            }
            Distinctness::DuplicationAllowed => Ok(generated_acir),
        }
    }
}

impl Context {
    fn new() -> Context {
        let mut acir_context = AcirContext::default();
        let current_side_effects_enabled_var = acir_context.add_constant(FieldElement::one());

        Context {
            ssa_values: HashMap::default(),
            current_side_effects_enabled_var,
            acir_context,
            initialized_arrays: HashSet::new(),
            memory_blocks: HashMap::default(),
            max_block_id: 0,
        }
    }

    /// Converts SSA into ACIR
    fn convert_ssa(
        self,
        ssa: Ssa,
        brillig: Brillig,
        last_array_uses: &HashMap<ValueId, InstructionId>,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let main_func = ssa.main();
        match main_func.runtime() {
            RuntimeType::Acir => self.convert_acir_main(main_func, &ssa, brillig, last_array_uses),
            RuntimeType::Brillig => self.convert_brillig_main(main_func, brillig),
        }
    }

    fn convert_acir_main(
        mut self,
        main_func: &Function,
        ssa: &Ssa,
        brillig: Brillig,
        last_array_uses: &HashMap<ValueId, InstructionId>,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];
        let input_witness = self.convert_ssa_block_params(entry_block.parameters(), dfg)?;

        for instruction_id in entry_block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg, ssa, &brillig, last_array_uses)?;
        }

        self.convert_ssa_return(entry_block.unwrap_terminator(), dfg)?;

        Ok(self.acir_context.finish(input_witness.collect()))
    }

    fn convert_brillig_main(
        mut self,
        main_func: &Function,
        brillig: Brillig,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let dfg = &main_func.dfg;

        let inputs = try_vecmap(dfg[main_func.entry_block()].parameters(), |param_id| {
            let typ = dfg.type_of_value(*param_id);
            self.create_value_from_type(&typ, &mut |this, _| Ok(this.acir_context.add_variable()))
        })?;
        let witness_inputs = self.acir_context.extract_witness(&inputs);

        let outputs: Vec<AcirType> =
            vecmap(main_func.returns(), |result_id| dfg.type_of_value(*result_id).into());

        let code = self.gen_brillig_for(main_func, &brillig)?;

        let output_values = self.acir_context.brillig(
            self.current_side_effects_enabled_var,
            code,
            inputs,
            outputs,
        )?;
        let output_vars: Vec<_> = output_values
            .iter()
            .flat_map(|value| value.clone().flatten())
            .map(|value| value.0)
            .collect();

        for acir_var in output_vars {
            self.acir_context.return_var(acir_var)?;
        }

        Ok(self.acir_context.finish(witness_inputs))
    }

    /// Adds and binds `AcirVar`s for each numeric block parameter or block parameter array element.
    fn convert_ssa_block_params(
        &mut self,
        params: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<RangeInclusive<u32>, RuntimeError> {
        // The first witness (if any) is the next one
        let start_witness = self.acir_context.current_witness_index().0 + 1;
        for param_id in params {
            let typ = dfg.type_of_value(*param_id);
            let value = self.convert_ssa_block_param(&typ)?;
            match &value {
                AcirValue::Var(_, _) => (),
                AcirValue::Array(values) => {
                    let block_id = self.block_id(param_id);
                    let v = vecmap(values, |v| v.clone());
                    self.initialize_array(block_id, values.len(), Some(&v))?;
                }
                AcirValue::DynamicArray(_) => unreachable!(
                    "The dynamic array type is created in Acir gen and therefore cannot be a block parameter"
                ),
            }
            self.ssa_values.insert(*param_id, value);
        }
        let end_witness = self.acir_context.current_witness_index().0;
        Ok(start_witness..=end_witness)
    }

    fn convert_ssa_block_param(&mut self, param_type: &Type) -> Result<AcirValue, RuntimeError> {
        self.create_value_from_type(param_type, &mut |this, typ| this.add_numeric_input_var(&typ))
    }

    fn create_value_from_type(
        &mut self,
        param_type: &Type,
        make_var: &mut impl FnMut(&mut Self, NumericType) -> Result<AcirVar, RuntimeError>,
    ) -> Result<AcirValue, RuntimeError> {
        match param_type {
            Type::Numeric(numeric_type) => {
                let typ = AcirType::new(*numeric_type);
                Ok(AcirValue::Var(make_var(self, *numeric_type)?, typ))
            }
            Type::Array(element_types, length) => {
                let mut elements = im::Vector::new();

                for _ in 0..*length {
                    for element in element_types.iter() {
                        elements.push_back(self.create_value_from_type(element, make_var)?);
                    }
                }

                Ok(AcirValue::Array(elements))
            }
            _ => unreachable!("ICE: Params to the program should only contains numbers and arrays"),
        }
    }

    /// Get the BlockId corresponding to the ValueId
    /// If there is no matching BlockId, we create a new one.
    fn block_id(&mut self, value: &ValueId) -> BlockId {
        if let Some(block_id) = self.memory_blocks.get(value) {
            return *block_id;
        }
        let block_id = BlockId(self.max_block_id);
        self.max_block_id += 1;
        self.memory_blocks.insert(*value, block_id);
        block_id
    }

    /// Creates an `AcirVar` corresponding to a parameter witness to appears in the abi. A range
    /// constraint is added if the numeric type requires it.
    ///
    /// This function is used not only for adding numeric block parameters, but also for adding
    /// any array elements that belong to reference type block parameters.
    fn add_numeric_input_var(
        &mut self,
        numeric_type: &NumericType,
    ) -> Result<AcirVar, RuntimeError> {
        let acir_var = self.acir_context.add_variable();
        if matches!(numeric_type, NumericType::Signed { .. } | NumericType::Unsigned { .. }) {
            self.acir_context.range_constrain_var(acir_var, numeric_type)?;
        }
        Ok(acir_var)
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        brillig: &Brillig,
        last_array_uses: &HashMap<ValueId, InstructionId>,
    ) -> Result<(), RuntimeError> {
        let instruction = &dfg[instruction_id];
        self.acir_context.set_call_stack(dfg.get_call_stack(instruction_id));
        match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self.convert_ssa_binary(binary, dfg)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let lhs = self.convert_value(*lhs, dfg);
                let rhs = self.convert_value(*rhs, dfg);

                fn get_var_equality_assertions(
                    lhs: AcirValue,
                    rhs: AcirValue,
                    read_from_index: &mut impl FnMut(BlockId, usize) -> Result<AcirVar, InternalError>,
                ) -> Result<Vec<(AcirVar, AcirVar)>, InternalError> {
                    match (lhs, rhs) {
                        (AcirValue::Var(lhs, _), AcirValue::Var(rhs, _)) => Ok(vec![(lhs, rhs)]),
                        (AcirValue::Array(lhs_values), AcirValue::Array(rhs_values)) => {
                            let var_equality_assertions = lhs_values
                                .into_iter()
                                .zip(rhs_values)
                                .map(|(lhs, rhs)| {
                                    get_var_equality_assertions(lhs, rhs, read_from_index)
                                })
                                .collect::<Result<Vec<_>, _>>()?
                                .into_iter()
                                .flatten()
                                .collect();
                            Ok(var_equality_assertions)
                        }
                        (
                            AcirValue::DynamicArray(AcirDynamicArray {
                                block_id: lhs_block_id,
                                len,
                            }),
                            AcirValue::DynamicArray(AcirDynamicArray {
                                block_id: rhs_block_id,
                                ..
                            }),
                        ) => try_vecmap(0..len, |i| {
                            let lhs_var = read_from_index(lhs_block_id, i)?;
                            let rhs_var = read_from_index(rhs_block_id, i)?;
                            Ok((lhs_var, rhs_var))
                        }),
                        _ => unreachable!("ICE: lhs and rhs should be of the same type"),
                    }
                }

                let mut read_dynamic_array_index =
                    |block_id: BlockId, array_index: usize| -> Result<AcirVar, InternalError> {
                        let index = AcirValue::Var(
                            self.acir_context.add_constant(FieldElement::from(array_index as u128)),
                            AcirType::NumericType(NumericType::NativeField),
                        );
                        let index_var = index.into_var()?;

                        self.acir_context.read_from_memory(block_id, &index_var)
                    };

                for (lhs, rhs) in
                    get_var_equality_assertions(lhs, rhs, &mut read_dynamic_array_index)?
                {
                    self.acir_context.assert_eq_var(lhs, rhs, assert_message.clone())?;
                }
            }
            Instruction::Cast(value_id, typ) => {
                let result_acir_var = self.convert_ssa_cast(value_id, typ, dfg)?;
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
                                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));

                                let code = self.gen_brillig_for(func, brillig)?;

                                let outputs: Vec<AcirType> = vecmap(result_ids, |result_id| dfg.type_of_value(*result_id).into());

                                let output_values = self.acir_context.brillig(self.current_side_effects_enabled_var, code, inputs, outputs)?;

                                // Compiler sanity check
                                assert_eq!(result_ids.len(), output_values.len(), "ICE: The number of Brillig output values should match the result ids in SSA");

                                for result in result_ids.iter().zip(output_values) {
                                    if let  AcirValue::Array(values) = &result.1 {
                                        let block_id = self.block_id(&dfg.resolve(*result.0));
                                        let values: Vec<AcirValue> = values.iter().cloned().collect();
                                        self.initialize_array(block_id, values.len(), Some(&values))?;
                                    }
                                    self.ssa_values.insert(*result.0, result.1);
                                }
                            }
                        }
                    }
                    Value::Intrinsic(intrinsic) => {
                        let outputs = self
                            .convert_ssa_intrinsic_call(*intrinsic, arguments, dfg, result_ids)?;

                        // Issue #1438 causes this check to fail with intrinsics that return 0
                        // results but the ssa form instead creates 1 unit result value.
                        // assert_eq!(result_ids.len(), outputs.len());

                        for (result, output) in result_ids.iter().zip(outputs) {
                            match &output {
                                // We need to make sure we initialize arrays returned from intrinsic calls
                                // or else they will fail if accessed with a dynamic index
                                AcirValue::Array(values) => {
                                    let block_id = self.block_id(result);
                                    let values = vecmap(values, |v| v.clone());

                                    self.initialize_array(block_id, values.len(), Some(&values))?;
                                }
                                AcirValue::DynamicArray(_) => {
                                    unreachable!("The output from an intrinsic call is expected to be a single value or an array but got {output:?}");
                                }
                                AcirValue::Var(_, _) => {
                                    // Do nothing
                                }
                            }
                            self.ssa_values.insert(*result, output);
                        }
                    }
                    Value::ForeignFunction(_) => unreachable!(
                        "All `oracle` methods should be wrapped in an unconstrained fn"
                    ),
                    _ => unreachable!("expected calling a function"),
                }
            }
            Instruction::Not(value_id) => {
                let (acir_var, typ) = match self.convert_value(*value_id, dfg) {
                    AcirValue::Var(acir_var, typ) => (acir_var, typ),
                    _ => unreachable!("NOT is only applied to numerics"),
                };
                let result_acir_var = self.acir_context.not_var(acir_var, typ)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                let result_acir_var =
                    self.convert_ssa_truncate(*value, *bit_size, *max_bit_size, dfg)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::EnableSideEffects { condition } => {
                let acir_var = self.convert_numeric_value(*condition, dfg)?;
                self.current_side_effects_enabled_var = acir_var;
            }
            Instruction::ArrayGet { .. } | Instruction::ArraySet { .. } => {
                self.handle_array_operation(instruction_id, dfg, last_array_uses)?;
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
        self.acir_context.set_call_stack(CallStack::new());
        Ok(())
    }

    fn gen_brillig_for(
        &self,
        func: &Function,
        brillig: &Brillig,
    ) -> Result<GeneratedBrillig, InternalError> {
        // Create the entry point artifact
        let mut entry_point = BrilligContext::new_entry_point_artifact(
            BrilligFunctionContext::parameters(func),
            BrilligFunctionContext::return_values(func),
            BrilligFunctionContext::function_id_to_function_label(func.id()),
        );
        // Link the entry point with all dependencies
        while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
            let artifact = &brillig.find_by_function_label(unresolved_fn_label.clone());
            let artifact = match artifact {
                Some(artifact) => artifact,
                None => {
                    return Err(InternalError::General {
                        message: format!("Cannot find linked fn {unresolved_fn_label}"),
                        call_stack: CallStack::new(),
                    })
                }
            };
            entry_point.link_with(artifact);
        }
        // Generate the final bytecode
        Ok(entry_point.finish())
    }

    /// Handles an ArrayGet or ArraySet instruction.
    /// To set an index of the array (and create a new array in doing so), pass Some(value) for
    /// store_value. To just retrieve an index of the array, pass None for store_value.
    fn handle_array_operation(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        last_array_uses: &HashMap<ValueId, InstructionId>,
    ) -> Result<(), RuntimeError> {
        // Pass the instruction between array methods rather than the internal fields themselves
        let (array, index, store_value) = match dfg[instruction] {
            Instruction::ArrayGet { array, index } => (array, index, None),
            Instruction::ArraySet { array, index, value, .. } => (array, index, Some(value)),
            _ => {
                return Err(InternalError::UnExpected {
                    expected: "Instruction should be an ArrayGet or ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into())
            }
        };

        let index_const = dfg.get_numeric_constant(index);

        match dfg.type_of_value(array) {
            Type::Array(_, _) => {
                match self.convert_value(array, dfg) {
                    AcirValue::Var(acir_var, _) => {
                        return Err(RuntimeError::InternalError(InternalError::UnExpected {
                            expected: "an array value".to_string(),
                            found: format!("{acir_var:?}"),
                            call_stack: self.acir_context.get_call_stack(),
                        }))
                    }
                    AcirValue::Array(array) => {
                        if let Some(index_const) = index_const {
                            let array_size = array.len();
                            let index = match index_const.try_to_u64() {
                                Some(index_const) => index_const as usize,
                                None => {
                                    let call_stack = self.acir_context.get_call_stack();
                                    return Err(RuntimeError::TypeConversion {
                                        from: "array index".to_string(),
                                        into: "u64".to_string(),
                                        call_stack,
                                    });
                                }
                            };
                            if index >= array_size {
                                // Ignore the error if side effects are disabled.
                                if self
                                    .acir_context
                                    .is_constant_one(&self.current_side_effects_enabled_var)
                                {
                                    let call_stack = self.acir_context.get_call_stack();
                                    return Err(RuntimeError::IndexOutOfBounds {
                                        index,
                                        array_size,
                                        call_stack,
                                    });
                                }
                                let result_type =
                                    dfg.type_of_value(dfg.instruction_results(instruction)[0]);
                                let value = self.create_default_value(&result_type)?;
                                self.define_result(dfg, instruction, value);
                                return Ok(());
                            }

                            let value = match store_value {
                                Some(store_value) => {
                                    let store_value = self.convert_value(store_value, dfg);
                                    AcirValue::Array(array.update(index, store_value))
                                }
                                None => array[index].clone(),
                            };

                            self.define_result(dfg, instruction, value);
                            return Ok(());
                        }
                    }
                    AcirValue::DynamicArray(_) => (),
                }
            }
            Type::Slice(_) => {
                // Do nothing we only want dynamic checks here
            }
            _ => unreachable!("ICE: expected array or slice type"),
        }

        let resolved_array = dfg.resolve(array);
        let map_array = last_array_uses.get(&resolved_array) == Some(&instruction);
        if store_value.is_some() {
            self.array_set(instruction, dfg, map_array)?;
        } else {
            self.array_get(instruction, array, index, dfg)?;
        }

        Ok(())
    }

    /// Generates a read opcode for the array
    fn array_get(
        &mut self,
        instruction: InstructionId,
        array: ValueId,
        index: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        let array = dfg.resolve(array);
        let block_id = self.block_id(&array);
        if !self.initialized_arrays.contains(&block_id) {
            match &dfg[array] {
                Value::Array { array, .. } => {
                    let values: Vec<AcirValue> =
                        array.iter().map(|i| self.convert_value(*i, dfg)).collect();
                    self.initialize_array(block_id, array.len(), Some(&values))?;
                }
                _ => {
                    return Err(RuntimeError::UnInitialized {
                        name: "array".to_string(),
                        call_stack: self.acir_context.get_call_stack(),
                    });
                }
            }
        }

        let index_var = self.convert_value(index, dfg).into_var()?;
        let read = self.acir_context.read_from_memory(block_id, &index_var)?;
        let typ = match dfg.type_of_value(array) {
            Type::Array(typ, _) => {
                if typ.len() != 1 {
                    // TODO(#2461)
                    unimplemented!(
                        "Non-const array indices is not implemented for non-homogenous array"
                    );
                }
                typ[0].clone()
            }
            Type::Slice(typ) => {
                if typ.len() != 1 {
                    // TODO(#2461)
                    unimplemented!(
                        "Non-const array indices is not implemented for non-homogenous array"
                    );
                }
                typ[0].clone()
            }
            _ => unreachable!("ICE - expected an array"),
        };
        let typ = AcirType::from(typ);
        self.define_result(dfg, instruction, AcirValue::Var(read, typ));
        Ok(())
    }

    /// Copy the array and generates a write opcode on the new array
    ///
    /// Note: Copying the array is inefficient and is not the way we want to do it in the end.
    fn array_set(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        map_array: bool,
    ) -> Result<(), InternalError> {
        // Pass the instruction between array methods rather than the internal fields themselves
        let (array, index, store_value, length) = match dfg[instruction] {
            Instruction::ArraySet { array, index, value, length } => (array, index, value, length),
            _ => {
                return Err(InternalError::UnExpected {
                    expected: "Instruction should be an ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                })
            }
        };

        // Fetch the internal SSA ID for the array
        let array = dfg.resolve(array);

        // Use the SSA ID to get or create its block ID
        let block_id = self.block_id(&array);

        // Every array has a length in its type, so we fetch that from
        // the SSA IR.
        let len = match dfg.type_of_value(array) {
            Type::Array(_, len) => len,
            Type::Slice(_) => {
                let length = length
                    .expect("ICE: array set on slice must have a length associated with the call");
                let length_acir_var = self.convert_value(length, dfg).into_var()?;
                let len = self.acir_context.var_to_expression(length_acir_var)?.to_const();
                let len = len
                    .expect("ICE: slice length should be fully tracked and constant by ACIR gen");
                len.to_u128() as usize
            }
            _ => unreachable!("ICE - expected an array"),
        };

        // Check if the array has already been initialized in ACIR gen
        // if not, we initialize it using the values from SSA
        let already_initialized = self.initialized_arrays.contains(&block_id);
        if !already_initialized {
            match &dfg[array] {
                Value::Array { array, .. } => {
                    let values: Vec<AcirValue> =
                        array.iter().map(|i| self.convert_value(*i, dfg)).collect();
                    self.initialize_array(block_id, array.len(), Some(&values))?;
                }
                _ => {
                    return Err(InternalError::General {
                        message: format!("Array {array} should be initialized"),
                        call_stack: self.acir_context.get_call_stack(),
                    })
                }
            }
        }

        // Since array_set creates a new array, we create a new block ID for this
        // array, unless map_array is true. In that case, we operate directly on block_id
        // and we do not create a new block ID.
        let result_id = dfg
            .instruction_results(instruction)
            .first()
            .expect("Array set does not have one result");
        let result_block_id;
        if map_array {
            self.memory_blocks.insert(*result_id, block_id);
            result_block_id = block_id;
        } else {
            // Initialize the new array with the values from the old array
            result_block_id = self.block_id(result_id);
            let init_values = try_vecmap(0..len, |i| {
                let index = AcirValue::Var(
                    self.acir_context.add_constant(FieldElement::from(i as u128)),
                    AcirType::NumericType(NumericType::NativeField),
                );
                let var = index.into_var()?;
                let read = self.acir_context.read_from_memory(block_id, &var)?;
                Ok(AcirValue::Var(read, AcirType::NumericType(NumericType::NativeField)))
            })?;
            self.initialize_array(result_block_id, len, Some(&init_values))?;
        }

        // Write the new value into the new array at the specified index
        let index_var = self.convert_value(index, dfg).into_var()?;
        let value_var = self.convert_value(store_value, dfg).into_var()?;
        self.acir_context.write_to_memory(result_block_id, &index_var, &value_var)?;

        let result_value =
            AcirValue::DynamicArray(AcirDynamicArray { block_id: result_block_id, len });
        self.define_result(dfg, instruction, result_value);
        Ok(())
    }

    /// Initializes an array with the given values and caches the fact that we
    /// have initialized this array.
    fn initialize_array(
        &mut self,
        array: BlockId,
        len: usize,
        values: Option<&[AcirValue]>,
    ) -> Result<(), InternalError> {
        self.acir_context.initialize_array(array, len, values)?;
        self.initialized_arrays.insert(array);
        Ok(())
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
    fn convert_ssa_return(
        &mut self,
        terminator: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) -> Result<(), InternalError> {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => unreachable!("ICE: Program must have a singular return"),
        };

        // The return value may or may not be an array reference. Calling `flatten_value_list`
        // will expand the array if there is one.
        let return_acir_vars = self.flatten_value_list(return_values, dfg);
        for acir_var in return_acir_vars {
            self.acir_context.return_var(acir_var)?;
        }
        Ok(())
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
            Value::ForeignFunction(_) => unimplemented!(
                "Oracle calls directly in constrained functions are not yet available."
            ),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value_id} {value:?}")
            }
        };
        self.ssa_values.insert(value_id, acir_value.clone());
        acir_value
    }

    fn convert_numeric_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, InternalError> {
        match self.convert_value(value_id, dfg) {
            AcirValue::Var(acir_var, _) => Ok(acir_var),
            AcirValue::Array(array) => Err(InternalError::UnExpected {
                expected: "a numeric value".to_string(),
                found: format!("{array:?}"),
                call_stack: self.acir_context.get_call_stack(),
            }),
            AcirValue::DynamicArray(_) => Err(InternalError::UnExpected {
                expected: "a numeric value".to_string(),
                found: "an array".to_string(),
                call_stack: self.acir_context.get_call_stack(),
            }),
        }
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        let lhs = self.convert_numeric_value(binary.lhs, dfg)?;
        let rhs = self.convert_numeric_value(binary.rhs, dfg)?;

        let binary_type = self.type_of_binary_operation(binary, dfg);
        match &binary_type {
            Type::Numeric(NumericType::Unsigned { bit_size })
            | Type::Numeric(NumericType::Signed { bit_size }) => {
                // Conservative max bit size that is small enough such that two operands can be
                // multiplied and still fit within the field modulus. This is necessary for the
                // truncation technique: result % 2^bit_size to be valid.
                let max_integer_bit_size = FieldElement::max_num_bits() / 2;
                if *bit_size > max_integer_bit_size {
                    return Err(RuntimeError::UnsupportedIntegerSize {
                        num_bits: *bit_size,
                        max_num_bits: max_integer_bit_size,
                        call_stack: self.acir_context.get_call_stack(),
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
            BinaryOp::Div => self.acir_context.div_var(
                lhs,
                rhs,
                binary_type,
                self.current_side_effects_enabled_var,
            ),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => self.acir_context.less_than_var(
                lhs,
                rhs,
                bit_count,
                self.current_side_effects_enabled_var,
            ),
            BinaryOp::Xor => self.acir_context.xor_var(lhs, rhs, binary_type),
            BinaryOp::And => self.acir_context.and_var(lhs, rhs, binary_type),
            BinaryOp::Or => self.acir_context.or_var(lhs, rhs, binary_type),
            BinaryOp::Mod => self.acir_context.modulo_var(
                lhs,
                rhs,
                bit_count,
                self.current_side_effects_enabled_var,
            ),
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
            (_, Type::Slice(..)) | (Type::Slice(..), _) => {
                unreachable!("Arrays are invalid in binary operations")
            }
            // If either side is a Field constant then, we coerce into the type
            // of the other operand
            (Type::Numeric(NumericType::NativeField), typ)
            | (typ, Type::Numeric(NumericType::NativeField)) => typ,
            // If either side is a numeric type, then we expect their types to be
            // the same.
            (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
                assert_eq!(lhs_type, rhs_type, "lhs and rhs types in {binary:?} are not the same");
                Type::Numeric(lhs_type)
            }
        }
    }

    /// Returns an `AcirVar` that is constrained to fit in the target type by truncating the input.
    /// If the target cast is to a `NativeField`, no truncation is required so the cast becomes a
    /// no-op.
    fn convert_ssa_cast(
        &mut self,
        value_id: &ValueId,
        typ: &Type,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        let (variable, incoming_type) = match self.convert_value(*value_id, dfg) {
            AcirValue::Var(variable, typ) => (variable, typ),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => {
                unreachable!("Cast is only applied to numerics")
            }
        };
        let target_numeric = match typ {
            Type::Numeric(numeric) => numeric,
            _ => unreachable!("Can only cast to a numeric"),
        };
        match target_numeric {
            NumericType::NativeField => {
                // Casting into a Field as a no-op
                Ok(variable)
            }
            NumericType::Unsigned { bit_size } => {
                if incoming_type.is_signed() {
                    todo!("Cast from unsigned to signed")
                }
                let max_bit_size = incoming_type.bit_size();
                if max_bit_size <= *bit_size {
                    // Incoming variable already fits into target bit size -  this is a no-op
                    return Ok(variable);
                }
                self.acir_context.truncate_var(variable, *bit_size, max_bit_size)
            }
            NumericType::Signed { .. } => todo!("Cast into signed"),
        }
    }

    /// Returns an `AcirVar`that is constrained to be result of the truncation.
    fn convert_ssa_truncate(
        &mut self,
        value_id: ValueId,
        bit_size: u32,
        max_bit_size: u32,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        let mut var = self.convert_numeric_value(value_id, dfg)?;
        match &dfg[value_id] {
            Value::Instruction { instruction, .. } => {
                if matches!(
                    &dfg[*instruction],
                    Instruction::Binary(Binary { operator: BinaryOp::Sub, .. })
                ) {
                    // Subtractions must first have the integer modulus added before truncation can be
                    // applied. This is done in order to prevent underflow.
                    let integer_modulus =
                        self.acir_context.add_constant(FieldElement::from(2_u128.pow(bit_size)));
                    var = self.acir_context.add_var(var, integer_modulus)?;
                }
            }
            Value::Param { .. } => {
                // Binary operations on params may have been entirely simplified if the operation
                // results in the identity of the parameter
            }
            _ => unreachable!(
                "ICE: Truncates are only ever applied to the result of a binary op or a param"
            ),
        };

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
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        match intrinsic {
            Intrinsic::BlackBox(black_box) => {
                // Slices are represented as a tuple of (length, slice contents).
                // We must check the inputs to determine if there are slices
                // and make sure that we pass the correct inputs to the black box function call.
                // The loop below only keeps the slice contents, so that
                // setting up a black box function with slice inputs matches the expected
                // number of arguments specified in the function signature.
                let mut arguments_no_slice_len = Vec::new();
                for (i, arg) in arguments.iter().enumerate() {
                    if matches!(dfg.type_of_value(*arg), Type::Numeric(_)) {
                        if i < arguments.len() - 1 {
                            if !matches!(dfg.type_of_value(arguments[i + 1]), Type::Slice(_)) {
                                arguments_no_slice_len.push(*arg);
                            }
                        } else {
                            arguments_no_slice_len.push(*arg);
                        }
                    } else {
                        arguments_no_slice_len.push(*arg);
                    }
                }

                let inputs = vecmap(&arguments_no_slice_len, |arg| self.convert_value(*arg, dfg));

                let output_count = result_ids.iter().fold(0usize, |sum, result_id| {
                    sum + dfg.try_get_array_length(*result_id).unwrap_or(1)
                });

                let vars = self.acir_context.black_box_function(black_box, inputs, output_count)?;

                Ok(Self::convert_vars_to_values(vars, dfg, result_ids))
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let radix = self.convert_value(arguments[1], dfg).into_var()?;
                let limb_size = self.convert_value(arguments[2], dfg).into_var()?;

                let result_type = Self::array_element_type(dfg, result_ids[1]);

                self.acir_context.radix_decompose(endian, field, radix, limb_size, result_type)
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let bit_size = self.convert_value(arguments[1], dfg).into_var()?;

                let result_type = Self::array_element_type(dfg, result_ids[1]);

                self.acir_context.bit_decompose(endian, field, bit_size, result_type)
            }
            Intrinsic::Sort => {
                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
                // We flatten the inputs and retrieve the bit_size of the elements
                let mut input_vars = Vec::new();
                let mut bit_size = 0;
                for input in inputs {
                    for (var, typ) in input.flatten() {
                        input_vars.push(var);
                        if bit_size == 0 {
                            bit_size = typ.bit_size();
                        } else {
                            assert_eq!(
                                bit_size,
                                typ.bit_size(),
                                "cannot sort element of different bit size"
                            );
                        }
                    }
                }
                // Generate the sorted output variables
                let out_vars = self
                    .acir_context
                    .sort(input_vars, bit_size, self.current_side_effects_enabled_var)
                    .expect("Could not sort");

                Ok(Self::convert_vars_to_values(out_vars, dfg, result_ids))
            }
            Intrinsic::ArrayLen => {
                let len = match self.convert_value(arguments[0], dfg) {
                    AcirValue::Var(_, _) => unreachable!("Non-array passed to array.len() method"),
                    AcirValue::Array(values) => (values.len() as u128).into(),
                    AcirValue::DynamicArray(array) => (array.len as u128).into(),
                };
                Ok(vec![AcirValue::Var(self.acir_context.add_constant(len), AcirType::field())])
            }
            Intrinsic::SlicePushBack => {
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);
                // TODO(#2461): make sure that we have handled nested struct inputs
                let element = self.convert_value(arguments[2], dfg);

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                new_slice.push_back(element);

                Ok(vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ])
            }
            Intrinsic::SlicePushFront => {
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);
                // TODO(#2461): make sure that we have handled nested struct inputs
                let element = self.convert_value(arguments[2], dfg);

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                new_slice.push_front(element);

                Ok(vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ])
            }
            Intrinsic::SlicePopBack => {
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                // TODO(#2461): make sure that we have handled nested struct inputs
                let elem = new_slice
                    .pop_back()
                    .expect("There are no elements in this slice to be removed");

                Ok(vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                    elem,
                ])
            }
            Intrinsic::SlicePopFront => {
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                // TODO(#2461): make sure that we have handled nested struct inputs
                let elem = new_slice
                    .pop_front()
                    .expect("There are no elements in this slice to be removed");

                Ok(vec![
                    elem,
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ])
            }
            Intrinsic::SliceInsert => {
                // Slice insert with a constant index
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);
                let index = self.convert_value(arguments[2], dfg).into_var()?;
                let element = self.convert_value(arguments[3], dfg);

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                // TODO(#2462): Slice insert is a little less obvious on how to implement due to the case
                // of having a dynamic index
                // The slice insert logic will need a more involved codegen
                let index = self.acir_context.var_to_expression(index)?.to_const();
                let index = index
                    .expect("ICE: slice length should be fully tracked and constant by ACIR gen");
                let index = index.to_u128() as usize;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                // TODO(#2461): make sure that we have handled nested struct inputs
                new_slice.insert(index, element);

                Ok(vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ])
            }
            Intrinsic::SliceRemove => {
                // Slice insert with a constant index
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice = self.convert_value(arguments[1], dfg);
                let index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                // TODO(#2462): allow slice remove with a constant index
                // Slice remove is a little less obvious on how to implement due to the case
                // of having a dynamic index
                // The slice remove logic will need a more involved codegen
                let index = self.acir_context.var_to_expression(index)?.to_const();
                let index = index
                    .expect("ICE: slice length should be fully tracked and constant by ACIR gen");
                let index = index.to_u128() as usize;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;
                // TODO(#2461): make sure that we have handled nested struct inputs
                let removed_elem = new_slice.remove(index);

                Ok(vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                    removed_elem,
                ])
            }
            _ => todo!("expected a black box function"),
        }
    }

    fn slice_intrinsic_input(
        &mut self,
        old_slice: &mut Vector<AcirValue>,
        input: AcirValue,
    ) -> Result<(), RuntimeError> {
        match input {
            AcirValue::Var(_, _) => {
                old_slice.push_back(input);
            }
            AcirValue::Array(vars) => {
                for var in vars {
                    self.slice_intrinsic_input(old_slice, var)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len }) => {
                for i in 0..len {
                    // We generate witnesses corresponding to the array values
                    let index = AcirValue::Var(
                        self.acir_context.add_constant(FieldElement::from(i as u128)),
                        AcirType::NumericType(NumericType::NativeField),
                    );

                    let index_var = index.into_var()?;
                    let value_read_var =
                        self.acir_context.read_from_memory(block_id, &index_var)?;
                    let value_read = AcirValue::Var(
                        value_read_var,
                        AcirType::NumericType(NumericType::NativeField),
                    );

                    old_slice.push_back(value_read);
                }
            }
        }
        Ok(())
    }

    /// Given an array value, return the numerical type of its element.
    /// Panics if the given value is not an array or has a non-numeric element type.
    fn array_element_type(dfg: &DataFlowGraph, value: ValueId) -> AcirType {
        match dfg.type_of_value(value) {
            Type::Array(elements, _) => {
                assert_eq!(elements.len(), 1);
                (&elements[0]).into()
            }
            Type::Slice(elements) => {
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
    /// If the type of a result id is an array, several acir vars are collected into
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

    /// Creates a default, meaningless value meant only to be a valid value of the given type.
    fn create_default_value(&mut self, param_type: &Type) -> Result<AcirValue, RuntimeError> {
        self.create_value_from_type(param_type, &mut |this, _| {
            Ok(this.acir_context.add_constant(FieldElement::zero()))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, rc::Rc};

    use acvm::{
        acir::{
            circuit::Opcode,
            native_types::{Expression, Witness},
        },
        FieldElement,
    };

    use crate::{
        brillig::Brillig,
        ssa::{
            function_builder::FunctionBuilder,
            ir::{function::RuntimeType, map::Id, types::Type},
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
        let array_type = Type::Array(element_type, 1);
        let array = builder.array_constant(im::Vector::unit(one), array_type);

        builder.terminate_with_return(vec![array]);

        let ssa = builder.finish();

        let context = Context::new();
        let mut acir = context.convert_ssa(ssa, Brillig::default(), &HashMap::default()).unwrap();

        let expected_opcodes =
            vec![Opcode::Arithmetic(&Expression::one() - &Expression::from(Witness(1)))];
        assert_eq!(acir.take_opcodes(), expected_opcodes);
        assert_eq!(acir.return_witnesses, vec![Witness(1)]);
    }
}
